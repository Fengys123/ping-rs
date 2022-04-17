use crate::{PingParam, PingResult};
use anyhow::Result;
use winping::{AsyncPinger, Buffer};

#[derive(Clone)]
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
        let mut joins = Vec::new();
        for index in 0..count as u32 {
            let self_clone = self.clone();
            let buffer_clone = buffer.clone();
            let join = tokio::spawn(async move {
                tokio::time::sleep(delay * index).await;
                let async_result = self_clone.pinger.send(addr, buffer_clone).await.result;
                (index, async_result)
            });
            joins.push(join);
        }
        let mut res = Vec::new();
        for join in joins {
            let (index, async_result) = join.await.unwrap();
            match async_result {
                Ok(time_spent) => res.push((index as usize, Some(time_spent as u128))),
                Err(_) => res.push((index as usize, None)),
            }
        }
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
            count: 5,
            delay: Duration::from_secs(5),
            expire: Duration::from_secs(5),
        };
        let res = win_pinger.unwrap().ping(ping_param).await;
        assert!(res.is_ok());

        let res = res.unwrap().0;
        assert_eq!(5, res.len());
        assert!(res[0].0 == 0 && res[0].1.is_some());
        assert!(res[1].0 == 1 && res[1].1.is_some());
    }
}
