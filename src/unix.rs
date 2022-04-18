use crate::{PingParam, PingResult};
use anyhow::Result;
use netdiag2::{Bind, Ping, Pinger};
use std::sync::Arc;

pub struct UnixPinger {
    pinger: Arc<Pinger>,
}

impl UnixPinger {
    pub async fn new() -> Result<Self> {
        let pinger = Arc::new(Pinger::new(&Bind::default()).await?);
        Ok(Self { pinger })
    }

    pub async fn ping(&self, ping_param: PingParam) -> Result<PingResult> {
        let PingParam {
            addr,
            count,
            delay,
            expire,
        } = ping_param;
        let mut res = Vec::new();
        let mut joins = Vec::new();

        for seq in 0..count {
            let p = self.pinger.clone();
            let join = tokio::spawn(async move {
                tokio::time::sleep(delay * seq as u32).await;
                let res = p.ping_once(addr, expire, seq.try_into().unwrap()).await;
                (seq, res)
            });
            joins.push(join);
        }
        for join in joins {
            let (seq, time_spent) = join.await?;
            match time_spent? {
                Some(time_spent) => res.push((seq, Some(time_spent.as_millis()))),
                None => res.push((seq, None)),
            }
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
            count: 5,
            delay: Duration::from_secs(1),
            expire: Duration::from_secs(5),
        };
        let res = unix_pinger.unwrap().ping(ping_param).await;
        assert!(res.is_ok());

        let res = res.unwrap().0;
        assert_eq!(5, res.len());
        for i in 0..5 {
            assert!(res[i].0 == i && res[i].1.is_some());
        }
    }
}
