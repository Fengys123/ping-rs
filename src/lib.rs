use anyhow::Result;
use netdiag::Ping;
use std::{net::IpAddr, time::Duration};

#[cfg(target_family = "unix")]
mod unix;
#[cfg(target_family = "windows")]
mod win;

#[derive(Debug)]
pub struct PingParam {
    pub addr: IpAddr,
    pub count: usize,
    pub delay: Duration,
    pub expiry: Duration,
}

impl From<PingParam> for Ping {
    fn from(ping_param: PingParam) -> Self {
        Self {
            addr: ping_param.addr,
            count: ping_param.count,
            expiry: ping_param.expiry,
        }
    }
}

pub type Seq = usize;
pub type TimeSpent = u128;
pub struct PingResult(Vec<(Seq, Option<TimeSpent>)>);

pub struct Pinger {
    #[cfg(target_family = "unix")]
    pub inner: unix::UnixPinger,
}

impl Pinger {
    pub async fn new() -> Self {
        #[cfg(target_family = "unix")]
        let pinger = unix::UnixPinger::new().await;
        Self { inner: pinger }
    }

    pub async fn ping(&self, ping_param: PingParam) -> Result<PingResult> {
        self.inner.ping(ping_param).await
    }
}

#[cfg(test)]
mod test {
    use crate::{PingParam, Pinger};
    use std::time::Duration;

    #[tokio::test]
    async fn test_ping_local() {
        let unix_pinger = Pinger::new().await;
        let ping_param = PingParam {
            addr: [127, 0, 0, 1].into(),
            count: 2,
            delay: Duration::from_secs(1),
            expiry: Duration::from_secs(5),
        };
        let res = unix_pinger.ping(ping_param).await;
        assert!(res.is_ok());

        let res = res.unwrap().0;
        assert_eq!(2, res.len());
        assert!(res[0].0 == 0 && res[0].1.is_some());
        assert!(res[1].0 == 1 && res[1].1.is_some());
    }
}
