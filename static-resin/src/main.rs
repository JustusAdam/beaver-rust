mod policy;
use std::fs::File;
use std::io::{Write, Error};
mod filter;
mod ResinBufWriter;
use crate::policy::Policied;

fn main() {
    println!("Hello, world!");
    let gp = policy::GradePolicy { studentId: "malte".to_string() };

    // make a protected grade object— see policy.rs for the impl of Policy on the grade
    let malte_grade = policy::Grade::make("malte".to_string(), 85, gp.clone()); 
    let kinan_grade = policy::Grade::make("kinan".to_string(), 87, gp.clone());

    let gp_copy = malte_grade.get_policy();

    // try and write to a file
    let mut f = File::create("malte").expect("Unable to create file");
    let ctxt = filter::FileContext {
        file_name: "malte".to_owned(), 
        path: "src/".to_owned(),
    };

    let mut bw = ResinBufWriter::ResinBufWriter::safe_create(f, filter::Context::File(ctxt));
    
    bw.safe_write(&malte_grade.get_studentId()); // this should return Ok(usize)
    bw.safe_write(&kinan_grade.get_studentId()); // this should return Err(PolicyErr)
}

// todo: flush out the use case (with filter objects), try to bypass it