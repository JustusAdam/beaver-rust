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

    pub fn map<V, F: Fn(T) -> V>(self, f: F) -> GPolicied<V> 
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

pub trait ExternalizePolicy {
    type Result;
    fn externalize_policy(self) -> Self::Result;
}

pub trait InternalizePolicy {
    type Result;
    fn internalize_policy(self) -> Self::Result;
}

pub trait AsPolicied : Sized {
    fn policied(self) -> GPolicied<Self> {
        GPolicied::make_default(self)
    }
    fn policied_with(self, policy: Box<dyn Policy>) -> GPolicied<Self> {
        GPolicied::make(self, policy)
    }
}

impl <T> AsPolicied for T {}

impl <T> ExternalizePolicy for Option<GPolicied<T>> 
{
    type Result = GPolicied<Option<T>>;
    fn externalize_policy(self) -> Self::Result {
        self.map(|p| p.map(Some)).unwrap_or_else(|| GPolicied::make_default(None))
    }
}


impl <T> ExternalizePolicy for Vec<GPolicied<T>> {
    type Result = GPolicied<Vec<T>>;
    fn externalize_policy(self) -> Self::Result {
        self.into_iter().fold(GPolicied::make_default(Vec::new()), |mut v, e| {
            v.push(e);
            v
        })
    }
}
impl <T> InternalizePolicy for GPolicied<Vec<T>> {
    type Result = Vec<GPolicied<T>>;
    fn internalize_policy(self) -> Self::Result {
        let GPolicied {policy, inner} = self;
        inner.into_iter().map(|v| GPolicied::make(v, policy.clone())).collect()
    }
}

pub trait InternalizePolicy_2_1 {
    type Result;
    fn internalize_policy_2_1(self) -> Self::Result;
}

impl <K,V> InternalizePolicy_2_1 for GPolicied<HashMap<K,V>> 
where K: Eq + std::hash::Hash
{
    type Result = HashMap<K, GPolicied<V>>;
    fn internalize_policy_2_1(self) -> Self::Result {
        let (inner, policy) = self.unsafe_decompose();
        inner.into_iter().map(|(k, v)| (k, v.policied_with(policy.clone()))).collect()
    }
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
    K : Eq + core::hash::Hash,
{
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
