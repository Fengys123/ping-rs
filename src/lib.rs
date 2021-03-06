use anyhow::Result;
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
    pub expire: Duration,
}
pub type Seq = usize;
pub type TimeSpent = u128;
pub struct PingResult(Vec<(Seq, Option<TimeSpent>)>);

pub struct Pinger {
    #[cfg(target_family = "unix")]
    pub inner: unix::UnixPinger,
    #[cfg(target_family = "windows")]
    pub inner: win::WinPinger,
}

#[cfg(target_family = "unix")]
impl Pinger {
    pub async fn new() -> Result<Self> {
        let pinger = unix::UnixPinger::new().await?;
        Ok(Self { inner: pinger })
    }

    pub async fn ping(&self, ping_param: PingParam) -> Result<PingResult> {
        self.inner.ping(ping_param).await
    }
}

#[cfg(target_family = "windows")]
impl Pinger {
    pub async fn new() -> Result<Self> {
        let pinger = win::WinPinger::new().await?;
        Ok(Self { inner: pinger })
    }

    pub async fn ping(&mut self, ping_param: PingParam) -> Result<PingResult> {
        self.inner.ping(ping_param).await
    }
}

#[cfg(test)]
mod test {
    use crate::{PingParam, Pinger};
    use std::time::Duration;

    #[tokio::test]
    async fn test_ping_local() {
        let pinger = Pinger::new().await;
        assert!(pinger.is_ok());

        let ping_param = PingParam {
            addr: [127, 0, 0, 1].into(),
            count: 5,
            delay: Duration::from_secs(1),
            expire: Duration::from_secs(5),
        };
        let res = pinger.unwrap().ping(ping_param).await;
        assert!(res.is_ok());

        let res = res.unwrap().0;
        assert_eq!(5, res.len());
        for i in 0..5 {
            assert!(res[i].0 == i && res[i].1.is_some());
        }
    }

    #[tokio::test]
    async fn test_ping_timeout() {
        let pinger = Pinger::new().await;
        assert!(pinger.is_ok());

        let ping_param = PingParam {
            addr: [244, 0, 0, 1].into(),
            count: 5,
            delay: Duration::from_secs(1),
            expire: Duration::from_secs(5),
        };
        let res = pinger.unwrap().ping(ping_param).await;
        assert!(res.is_ok());

        let res = res.unwrap().0;
        assert_eq!(5, res.len());
        for i in 0..5 {
            assert!(res[i].0 == i && res[i].1.is_none());
        }
    }
}
