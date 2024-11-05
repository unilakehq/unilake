use casbin::{EffectKind, Effector, EffectorStream};

#[derive(Clone)]
pub struct PdpEffectStream {
    done: bool,
    idx: usize,
    cap: usize,
    res: bool,
    app: bool,
    expl: Vec<usize>,
}

#[derive(Default)]
pub struct PdpEffector;

impl Effector for PdpEffector {
    fn new_stream(&self, _expr: &str, cap: usize) -> Box<dyn EffectorStream> {
        assert!(cap > 0);
        Box::new(PdpEffectStream {
            done: false,
            cap,
            idx: 0,
            expl: Vec::with_capacity(10),
            res: false,
            app: false,
        })
    }
}

impl PdpEffectStream {
    fn push_explain(&mut self, idx: usize) {
        if self.cap > 0 {
            self.expl.push(idx);
        }
    }
}

impl EffectorStream for PdpEffectStream {
    #[inline]
    fn next(&self) -> bool {
        assert!(self.done);
        self.res
    }

    #[inline]
    fn explain(&self) -> Option<Vec<usize>> {
        assert!(self.done);
        if self.expl.is_empty() {
            None
        } else {
            Some(self.expl.clone())
        }
    }

    fn push_effect(&mut self, eft: EffectKind) -> bool {
        if eft == EffectKind::Deny {
            self.res = false;
            self.done = true;
            self.push_explain(self.idx)
        } else if eft == EffectKind::Approval {
            self.app = true;
            self.push_explain(self.idx);
        } else if eft == EffectKind::Approved && self.app {
            self.res = true;
            self.done = true;
            self.push_explain(self.idx);
        } else if eft == EffectKind::Allow {
            self.res = true;
            self.push_explain(self.idx);
        }

        if self.idx + 1 == self.cap {
            self.done = true;
            self.idx = self.cap;
        } else {
            self.idx += 1;
        }

        self.done
    }
}
