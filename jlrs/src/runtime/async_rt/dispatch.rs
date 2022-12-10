use std::{fmt::Debug, marker::PhantomData};

use crate::{
    async_util::affinity::{Affinity, ToAny, ToMain, ToWorker},
    runtime::async_rt::{queue::Sender, Message},
};

pub struct Dispatch<'a, D> {
    msg: Message,
    sender: &'a Sender<Message>,
    _dispatch: PhantomData<D>,
}

impl<'a, D> Debug for Dispatch<'a, D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Dispatch").finish()
    }
}

impl<'a, D: Affinity> Dispatch<'a, D> {
    pub(crate) fn new(sender: &'a Sender<Message>, msg: Message) -> Self {
        Dispatch {
            msg,
            sender,
            _dispatch: PhantomData,
        }
    }
}

impl<'a, D: ToAny> Dispatch<'a, D> {
    pub async fn dispatch_any(self) {
        self.sender.send(self.msg).await
    }

    pub fn try_dispatch_any(self) -> Result<(), Self> {
        if let Some(msg) = self.sender.try_send_or_return(self.msg) {
            Err(Dispatch {
                msg,
                sender: self.sender,
                _dispatch: PhantomData,
            })
        } else {
            Ok(())
        }
    }
}

impl<'a, D: ToMain> Dispatch<'a, D> {
    pub async fn dispatch_main(self) {
        self.sender.send_main(self.msg).await
    }

    pub fn try_dispatch_main(self) -> Result<(), Self> {
        if let Some(msg) = self.sender.try_send_main_or_return(self.msg) {
            Err(Dispatch {
                msg,
                sender: self.sender,
                _dispatch: PhantomData,
            })
        } else {
            Ok(())
        }
    }
}

impl<'a, D: ToWorker> Dispatch<'a, D> {
    pub async fn dispatch_worker(self) {
        self.sender.send_worker(self.msg).await
    }

    pub fn try_dispatch_worker(self) -> Result<(), Self> {
        if let Some(msg) = self.sender.try_send_worker_or_return(self.msg) {
            Err(Dispatch {
                msg,
                sender: self.sender,
                _dispatch: PhantomData,
            })
        } else {
            Ok(())
        }
    }
}
