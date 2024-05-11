use core::{future::Future, pin::Pin, task::Poll};

pub fn with_condition<A, B>(cond_signal: A, exec: B) -> WithCondition<A, B>
where
    A: Future<Output = bool>,
    B: Future,
{
    WithCondition {
        cond_signal,
        exec,
        state: CondState::Running,
    }
}

pub struct WithCondition<A, B> {
    cond_signal: A,
    exec: B,
    state: CondState,
}

pub enum CondState {
    Running,
    Waiting,
}


impl<A, B> Future for WithCondition<A, B>
where
    A: Future<Output = bool>,
    B: Future,
{
    type Output = B::Output;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        let sig = unsafe { Pin::new_unchecked(&mut this.cond_signal) };
        let exec = unsafe { Pin::new_unchecked(&mut this.exec) };

        match this.state {
            CondState::Running => {
                match sig.poll(cx) {
                    Poll::Ready(running) => {
                        // the signal is sent
                        if running {
                            this.state = CondState::Running;
                            return exec.poll(cx);
                        } else {
                            this.state = CondState::Waiting;
                            return Poll::Pending;
                        }
                    }
                    Poll::Pending => {
                        // the signal is not available, i.e. not sent
                        // thus we are free to run the exec future
                        return exec.poll(cx);
                    }
                }
            }
            CondState::Waiting => match sig.poll(cx) {
                Poll::Ready(running) if running => {
                    // resume signal
                    this.state = CondState::Running;
                    return exec.poll(cx);
                }
                // keep waiting
                _ => Poll::Pending,
            },
        }
    }
}