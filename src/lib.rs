
pub mod runtime;
pub mod arrow;

#[cfg(test)]
mod tests {

    use arrow::{Arrow};
    use arrow::prim::{map,pause,product};

    #[test]
    fn test_pause () {
        product (
            map (|()| { println!("foo"); })
            .bind (pause ())
            .bind (map (|()| { println!("bar"); }))
        ,   map (|()| { println!("foo"); })
            .bind (pause ())
            .bind (map (|()| { println!("bar"); })) 
        )
        .execute (((),()));
    }

}

