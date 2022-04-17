use crate::{PingParam, PingResult};
use anyhow::Result;
use futures::{pin_mut, StreamExt};
use netdiag::{Bind, Ping, Pinger};
use tokio::time::sleep;

pub struct UnixPinger {
    pinger: Pinger,
}

impl UnixPinger {
    pub async fn new() -> Result<Self> {
        let pinger = Pinger::new(&Bind::default()).await?;
        Ok(Self { pinger })
    }
}

impl UnixPinger {
    pub async fn ping(&self, ping_param: PingParam) -> Result<PingResult> {
        let delay = ping_param.delay;
        let ping = ping_param.into();
        let stream = self.pinger.ping(&ping).enumerate();
        pin_mut!(stream);

        let mut res = Vec::new();
        while let Some((seq, time_spent)) = stream.next().await {
            match time_spent? {
                Some(time_spent) => res.push((seq, Some(time_spent.as_micros()))),
                None => res.push((seq, None)),
            }
            sleep(delay).await;
        }
        Ok(PingResult(res))
    }
}

impl From<PingParam> for Ping {
    fn from(ping_param: PingParam) -> Self {
        Self {
            addr: ping_param.addr,
            count: ping_param.count,
            expiry: ping_param.expire,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{unix::UnixPinger, PingParam};
    use std::time::Duration;

    #[tokio::test]
    async fn test_unix_ping() {
        let unix_pinger = UnixPinger::new().await;
        assert!(unix_pinger.is_ok());

        let ping_param = PingParam {
            addr: [127, 0, 0, 1].into(),
            count: 2,
            delay: Duration::from_secs(1),
            expire: Duration::from_secs(5),
        };
        let res = unix_pinger.unwrap().ping(ping_param).await;
        assert!(res.is_ok());

        let res = res.unwrap().0;
        println!("{:?}", res);
        assert_eq!(2, res.len());
        assert!(res[0].0 == 0 && res[0].1.is_some());
        assert!(res[1].0 == 1 && res[1].1.is_some());
    }
}
