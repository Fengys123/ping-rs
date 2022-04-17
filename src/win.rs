use crate::{PingParam, PingResult};
use anyhow::Result;
use winping::{AsyncPinger, Buffer};

pub struct WinPinger {
    pinger: AsyncPinger,
}

impl WinPinger {
    pub async fn new() -> Result<Self> {
        let pinger = AsyncPinger::new();
        Ok(Self { pinger })
    }

    pub async fn ping(&mut self, ping_param: PingParam) -> Result<PingResult> {
        let PingParam {
            addr,
            count,
            delay,
            expire,
        } = ping_param;
        self.pinger.set_timeout(expire.as_millis() as u32);
        let buffer = Buffer::new();

        let mut res = Vec::new();
        for index in 0..count {
            let async_result = self.pinger.send(addr, buffer.clone()).await.result;
            match async_result {
                Ok(time_spent) => res.push((index, Some(time_spent as u128))),
                Err(_) => res.push((index, None)),
            }
            tokio::time::sleep(delay).await;
        }
        println!("res: {:?}", res);
        Ok(PingResult(res))
    }
}

#[cfg(test)]
mod tests {
    use crate::{win::WinPinger, PingParam};
    use std::time::Duration;

    #[tokio::test]
    async fn test() {
        let win_pinger = WinPinger::new().await;
        assert!(win_pinger.is_ok());

        let ping_param = PingParam {
            addr: [127, 0, 0, 1].into(),
            count: 2,
            delay: Duration::from_secs(1),
            expire: Duration::from_secs(5),
        };
        let res = win_pinger.unwrap().ping(ping_param).await;
        assert!(res.is_ok());

        let res = res.unwrap().0;
        assert_eq!(2, res.len());
        assert!(res[0].0 == 0 && res[0].1.is_some());
        assert!(res[1].0 == 1 && res[1].1.is_some());
    }
}
