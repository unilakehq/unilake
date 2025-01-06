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

#[cfg(test)]
mod tests {
    use crate::effector::PdpEffectStream;
    use casbin::{EffectKind, EffectorStream};

    fn get_sut(max: usize) -> PdpEffectStream {
        PdpEffectStream {
            done: false,
            idx: 0,
            cap: max,
            res: false,
            app: false,
            expl: vec![],
        }
    }

    #[test]
    fn test_pdp_effect_deny() {
        let mut sut = get_sut(4);
        sut.push_effect(EffectKind::Indeterminate);
        sut.push_effect(EffectKind::Allow);
        sut.push_effect(EffectKind::Deny);
        sut.push_effect(EffectKind::Indeterminate);

        assert_eq!(sut.next(), false);
        assert_eq!(sut.explain(), Some(vec![1, 2]));
    }

    #[test]
    fn test_pdp_effect_allowed() {
        let mut sut = get_sut(3);
        sut.push_effect(EffectKind::Indeterminate);
        sut.push_effect(EffectKind::Allow);
        sut.push_effect(EffectKind::Indeterminate);

        assert_eq!(sut.next(), true);
        assert_eq!(sut.explain(), Some(vec![1]));
    }

    #[test]
    fn test_pdp_effect_approval() {
        let mut sut = get_sut(3);
        sut.push_effect(EffectKind::Indeterminate);
        sut.push_effect(EffectKind::Approval);
        sut.push_effect(EffectKind::Indeterminate);

        assert_eq!(sut.next(), false);
        assert_eq!(sut.explain(), Some(vec![1]));
    }

    #[test]
    fn test_pdp_effect_approved() {
        let mut sut = get_sut(5);
        assert!(!sut.push_effect(EffectKind::Indeterminate));
        assert!(!sut.push_effect(EffectKind::Approval));
        assert!(!sut.push_effect(EffectKind::Indeterminate));
        assert!(!sut.push_effect(EffectKind::Approved));
        assert!(sut.push_effect(EffectKind::Indeterminate));

        assert_eq!(sut.next(), true);
        assert_eq!(sut.explain(), Some(vec![1, 3]));
    }

    #[test]
    fn test_pdp_effect_approved_but_denied() {
        let mut sut = get_sut(6);
        assert!(!sut.push_effect(EffectKind::Indeterminate));
        assert!(!sut.push_effect(EffectKind::Approval));
        assert!(!sut.push_effect(EffectKind::Indeterminate));
        assert!(!sut.push_effect(EffectKind::Approved));
        assert!(!sut.push_effect(EffectKind::Indeterminate));
        assert!(sut.push_effect(EffectKind::Deny));

        assert_eq!(sut.next(), false);
        assert_eq!(sut.explain(), Some(vec![1, 3, 5]));
    }

    #[test]
    fn test_pdp_effect_unknown() {
        let mut sut = get_sut(3);
        sut.push_effect(EffectKind::Indeterminate);
        sut.push_effect(EffectKind::Indeterminate);
        sut.push_effect(EffectKind::Indeterminate);

        assert_eq!(sut.next(), false);
        assert_eq!(sut.explain(), None);
    }
}
