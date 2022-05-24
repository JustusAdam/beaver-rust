use std::collections::HashMap;
use crate::policy::{Policied, NonePolicy, Policy, PolicyError};

#[derive(Deserialize, Clone)]
pub struct GPolicied<T> {
    inner: T,
    policy: Box<dyn Policy>
}


// Could also make a "PoliciedRef" that holds a reference to the policy instead
impl <T> GPolicied<T> {
    pub fn as_ref(&self) -> GPolicied<&T> {
        GPolicied::make(&self.inner, self.policy.clone())
    }
    pub fn make_default(inner: T) -> Self 
    where T: Clone
    {
        Policied::make(inner, Box::new(NonePolicy))
    }

    pub fn apply<X, V>(self, x : GPolicied<X>) -> GPolicied<V> 
    where
        T: Fn(X) -> V,
        V: Clone
    {
        let GPolicied { inner, policy } = self;
        let GPolicied { inner: x, policy: p2 } = x;
        GPolicied::make(inner(x), policy.merge(&p2).unwrap())
    }

    pub fn map<V: serde::Serialize + Clone, F: Fn(T) -> V>(self, f: F) -> GPolicied<V> 
    {
        let GPolicied { inner, policy } = self;
        GPolicied::make(f(inner), policy)
    }

    pub fn unsafe_into_inner(self) -> T {
        self.inner
    }

    pub fn unsafe_decompose(self) -> (T, Box<dyn Policy>) {
        (self.inner,self.policy)
    }

    pub fn unsafe_borrow_inner(&self) -> &T {
        &self.inner
    }

    pub fn unsafe_borrow_inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    pub fn unsafe_borrow_decompose(&self) -> (&T, &Box<dyn Policy>) {
        (&self.inner, &self.policy)
    }
    pub fn unsafe_borrow_decompose_mut(& mut self) -> (&mut T, &mut Box<dyn Policy>) {
        (&mut self.inner, &mut self.policy)
    }
}

pub fn internalize_option<T : serde::Serialize + Clone>(o: Option<GPolicied<T>>) -> GPolicied<Option<T>> {
    o.map(|p| p.map(Some)).unwrap_or_else(|| GPolicied::make_default(None))
}

pub fn internalize_vec<T: serde::Serialize + Clone>(v : Vec<GPolicied<T>>) -> GPoliciedVec<T> {
    v.into_iter().fold(GPoliciedVec::new(), |mut v, e| {
        v.push(e);
        v
    })
}

impl <T> Policied<T> for GPolicied<T> 
{
    fn make(inner: T, policy: Box<dyn Policy>) -> Self {
        Self {
            inner, policy
        }
    }
    fn get_policy(&self) -> &Box<dyn Policy> {
        &self.policy
    }
    fn remove_policy(&mut self) -> () { self.policy = Box::new(NonePolicy); }
    fn export_check(self, ctxt: &crate::filter::Context) -> Result<T, PolicyError> 
    {
        self.get_policy().check(&ctxt).and_then(|_| Ok(self.inner))
    }
    fn export_check_borrow(&self, ctxt: &crate::filter::Context) -> Result<&T, PolicyError> 
    {
        self.get_policy().check(&ctxt).and_then(|_| Ok(&self.inner))
    }
    fn unsafe_export(self) -> T 
    {
        self.inner
    }
}

pub type GPoliciedVec<T> = GPolicied<Vec<T>>;

impl <T> GPoliciedVec<T> {
    pub fn new() -> Self 
    where T: serde::Serialize + Clone
    {
        GPolicied::make_default(Vec::new())
    }
    pub fn push(&mut self, e: GPolicied<T>) {
        let GPolicied { policy, inner } = e;
        self.policy.merge(&policy).unwrap();
        self.inner.push(inner);
    }

    pub fn pop(&mut self) -> Option<GPolicied<T>> {
        self.inner.pop().map(|e| GPolicied { inner: e, policy: self.policy.clone() })
    }
}

pub type PoliciedValHashMap<K, V> = GPolicied<HashMap<K,V>>;

impl <K,V> PoliciedValHashMap<K, V> 
where
    K : serde::Serialize + Clone + Eq + core::hash::Hash,
    V : serde::Serialize + Clone,
{
    pub fn new() -> Self {
        GPolicied::make_default(HashMap::new())
    }
    pub fn insert(&mut self, k: K, v: GPolicied<V>) -> Option<GPolicied<V>> {
        let GPolicied { policy, inner } = v;
        let ret = self.inner.insert(k, inner).map(|r| GPolicied::make(r, self.policy.clone()));
        self.policy = self.policy.merge(&policy).unwrap();
        ret
    }
    pub fn get(&self, k: &K) -> Option<GPolicied<&V>> {
        self.inner.get(k).map(|v| GPolicied::make(v, self.policy.clone()))
    }
    pub fn insert_kv(&mut self, kv: GPolicied<(K, V)>) -> Option<GPolicied<V>> {
        let GPolicied { policy, inner: (k, v) } = kv;
        let ret = self.inner.insert(k, v).map(|r| GPolicied::make(r, self.policy.clone()));
        self.policy = self.policy.merge(&policy).unwrap();
        ret
    }
}
