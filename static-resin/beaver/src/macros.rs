#[macro_export]
macro_rules! derive_policied {
    ($input_type:ty, $output_type_name:ident) => {
        derive_policied!($input_type, $output_type_name ,);
    };
    ($input_type:ty, $output_type_name:ident, $($ty_vars:ident),*) => {
        #[derive(Serialize, Deserialize, Clone)]
        pub struct $output_type_name<$($ty_vars),*> {
            inner: $input_type,
            policy: Box<dyn $crate::policy::Policy>
        }

        impl <$($ty_vars),*> $crate::policy::Policied<$input_type> for $output_type_name<$($ty_vars),*> {
            fn make(inner: $input_type, policy: Box<dyn $crate::policy::Policy>) -> Self {
                Self {
                    inner, policy
                }
            }
            fn get_policy(&self) -> &Box<dyn $crate::policy::Policy> {
                &self.policy
            }
            fn remove_policy(&mut self) -> () { self.policy = Box::new($crate::policy::NonePolicy); }
            fn export_check(self, ctxt: &$crate::filter::Context) -> Result<$input_type, $crate::policy::PolicyError> {
                match self.get_policy().check(&ctxt) {
                    Ok(_) => {
                        Ok(self.inner)
                    }, 
                    Err(pe) => { Err(pe) }
                }
            }    
            fn export_check_borrow(&self, ctxt: &$crate::filter::Context) -> Result<&$input_type, $crate::policy::PolicyError> {
                match self.get_policy().check(&ctxt) {
                    Ok(_) => {
                        Ok(&self.inner)
                    }, 
                    Err(pe) => { Err(pe) }
                }
            }
            fn unsafe_export(self) -> $input_type {
                self.inner
            }
        }
    };
}

/*
This macro can be used to easily create a simple policied version of a type. 
It creates a PoliciedTYPE struct and implements the Policied<TYPE> trait for it.

Usage: derive_policied!(TYPE, PoliciedTYPE);
*/

/*
This macro can be used to easily create a policied vector that has elements of a 
a policied type, where the policy on the vector is a merges version of the policies
on its elements. 
It creates a PoliciedTYPEVec struct and implements various vector operations.
PoliciedTYPE must have been previously derived.

Usage: derive_policied_vec!(PoliciedTYPEVec, TYPE, PoliciedTYPE);
*/
#[macro_export]
macro_rules! derive_policied_vec {
    ($policied_vector_type:ident, $unpolicied_element_type:ty, $policied_element_type:ident) => {
        $crate::derive_policied!(Vec<$unpolicied_element_type>, $policied_vector_type);

        impl $policied_vector_type {
            pub fn push(&mut self, value: $unpolicied_element_type) {
                self.inner.push(value);
            }
        
            pub fn push_policy(&mut self, value: $policied_element_type) {
                self.policy = self.policy.merge(value.get_policy()).unwrap();
                self.inner.push(value.unsafe_export());
            }
        
            pub fn pop(&mut self) -> Option<$policied_element_type> {
                match self.inner.pop() {
                    Some(v) => Some($policied_element_type { inner: v, policy: self.policy.clone() }),
                    None => None
                }
            }

            pub fn sort_by<F>(&mut self, compare: F) where F: FnMut(&$unpolicied_element_type, &$unpolicied_element_type) -> std::cmp::Ordering, {
                self.inner.sort_by(compare)
            }
        }
    }
}

#[macro_export]
macro_rules! derive_policied_map {
    ($policied_map_type:ident, $key_type:ty, $unpolicied_element_type:ty, $policied_element_type:ident) => {
        $crate::derive_policied!(std::collections::HashMap<$key_type,$unpolicied_element_type>, $policied_map_type);

        impl $policied_map_type {
            pub fn insert(&mut self, key: $key_type, v: $policied_element_type) -> Option<$policied_element_type> {
                self.policy = self.policy.merge(v.get_policy()).unwrap();
                self.inner.insert(key, v.export());
            }

            pub fn get(&mut self, key: &$key_type, v: $policied_element_type) -> Option<&$policied_element_type> {
                self.inner.get(key).map(|v| $policied_element_type { inner: v, policy: self.policy.clone()})
            }

            pub fn new() -> Self {
                Self { policy: $crate::policy::NonePolicy, inner: std::collections::HashMap::new() }
            }
        }
    }
}

/*
This macro can be used to easily create a policied option that may contain a policied type. 
It creates a PoliciedTYPEOption struct and methods to create and obtain the option.
PoliciedTYPE must have been previously derived.

Usage: derive_policied_option!(PoliciedTYPEOption, TYPE, PoliciedTYPE);
*/
#[macro_export]
macro_rules! derive_policied_option {
    ($policied_option_type:ident, $unpolicied_element_type:ty, $policied_element_type:ident) => {
        $crate::derive_policied!(Option<$unpolicied_element_type>, $policied_option_type);
        
        impl $policied_option_type {
            pub fn make_option(ops: Option<$policied_element_type>) -> Self {
                match ops {
                    Some(s) => $policied_option_type {
                        policy: s.policy.clone(),
                        inner: Some(s.unsafe_export()),
                    },
                    None => $policied_option_type {
                        inner: None,
                        policy: Box::new($crate::policy::NonePolicy)
                    }
                }
            }
        
            pub fn get_option(self) -> Option<$policied_element_type> {
                match self.inner {
                    Some(s) => Some($policied_element_type::make(s, self.policy.clone())),
                    None => None
                }
            }
        }
    }
}