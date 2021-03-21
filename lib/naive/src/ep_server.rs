use alloc::sync::Arc;
use core::sync::atomic::{AtomicUsize, Ordering};

use conquer_once::spin::OnceCell;
use hashbrown::HashMap;
use spin::{Mutex, MutexGuard};

use crate::space_manager::gsm;
use rustyl4api::ipc::IpcMessage;
use crate::objects::EpCap;

pub struct Ep {
    ep: EpCap,
    cur_badge: AtomicUsize,
}

impl Ep {
    pub const fn from_unbadged(ep: EpCap) -> Self {
        Self {
            ep,
            cur_badge: AtomicUsize::new(100),
        }
    }

    pub fn derive_badged_cap(&self) -> Option<(usize, EpCap)> {
        let slot = gsm!().cspace_alloc().unwrap();
        let badge = self.cur_badge.fetch_add(1, Ordering::Relaxed);
        self.ep.mint(slot, badge).unwrap();
        Some((badge, EpCap::new(slot)))
    }
}

pub struct EpServer {
    event_handlers: Mutex<HashMap<usize, Arc<dyn EpMsgHandler>>>,
    ntf_handler: Mutex<[Option<Arc<dyn EpNtfHandler>>; 64]>,
    ep: Ep,
}

impl EpServer {
    pub fn new(ep: EpCap) -> Self {
        Self {
            ep: Ep::from_unbadged(ep),
            event_handlers: Mutex::new(HashMap::new()),
            ntf_handler: Mutex::new([None; 64]),
        }
    }

    fn get_event_handlers(&self) -> MutexGuard<HashMap<usize, Arc<dyn EpMsgHandler>>> {
        self.event_handlers
            .lock()
    }

    pub fn derive_badged_cap(&self) -> Option<(usize, EpCap)> {
        self.ep.derive_badged_cap()
    }

    pub fn insert_event<T: 'static + EpMsgHandler>(&self, badge: usize, cb: T) {
        self.get_event_handlers().insert(badge, Arc::new(cb));
    }

    pub fn remove_event(&self, badge: usize) {
        self.get_event_handlers().remove(&badge);
    }

    pub fn insert_notification<T: 'static + EpNtfHandler>(&self, ntf: usize, cb: T) {
        self.ntf_handler.lock()[ntf] = Some(Arc::new(cb));
    }

    pub fn run(&self) {
        let mut recv_slot = gsm!().cspace_alloc().unwrap();
        loop {
            let ret = self.ep.ep.receive(Some(recv_slot));
            match ret {
                Ok(IpcMessage::Message {
                    payload: _,
                    payload_len: _,
                    need_reply: _,
                    cap_transfer,
                    badge,
                }) => {
                    if let Some(b) = badge {
                        let cb = self.get_event_handlers().get(&b).map(|cb| cb.clone());
                        if let Some(cb) = cb {
                            let cap_trans = if cap_transfer { Some(recv_slot) } else { None };
                            cb.handle_ipc(self, ret.unwrap(), cap_trans);
                        } else {
                            kprintln!("warning: receive message from unhandled badge {}", b);
                        }
                    } else {
                        kprintln!("warning: receive unbadged message");
                    }
                    //TOO: leak previous alloced slot now. should find some other way...
                    if cap_transfer {
                        recv_slot = gsm!().cspace_alloc().unwrap();
                    }
                }
                Ok(IpcMessage::Notification(ntf_mask)) => {
                    let mut ntf_mask = ntf_mask;
                    while ntf_mask.trailing_zeros() != 64 {
                        let ntf = ntf_mask.trailing_zeros() as usize;
                        let cb = &self.ntf_handler.lock()[ntf];
                        if let Some(c) = cb {
                            c.handle_notification(self, ntf);
                        }
                        ntf_mask &= !(1 << ntf);
                    }
                }
                e => {
                    kprintln!("e {:?}", e);
                }
            }
        }
    }
}

pub trait EpMsgHandler: Send + Sync {
    fn handle_ipc(
        &self,
        _ep_server: &EpServer,
        _msg: IpcMessage,
        _cap_transfer_slot: Option<usize>,
    ) {
    }

    fn handle_fault(&self) {}
}

pub trait EpNtfHandler: Send + Sync {
    fn handle_notification(&self, _ep_server: &EpServer, _ntf: usize) {}
}

pub static EP_SERVER: OnceCell<EpServer> = OnceCell::uninit();
pub fn ep_server() -> &'static EpServer {
    EP_SERVER.get().unwrap()
}
