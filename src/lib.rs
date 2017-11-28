
pub mod runtime;
pub mod arrow;
pub mod signal;

#[cfg(test)]
mod tests {

    use std::result::{Result};

    use arrow::{Arrow};
    use arrow::prim::{identity,map,pause,fixpoint,product,fork};

    use signal::{Signal};
    use signal::prim::{PureSignal};

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

    #[test]
    fn test_pure_signal () {
        let s = PureSignal::new ();
        let p1 = s.emit ()
        .bind  ( pause () )
        .bind  ( s.emit () )
        .bind  ( pause () )
        .bind  ( pause () )
        ;
        let p2 = {
            let p = s.present (
                map (|()| { println! ("present"); } )
                .bind ( pause () )
                .bind ( map (|()| { Result::Ok (()) }) )
            ,   map (|()| { println! ("not present"); Result::Err (()) } )
            );
            fixpoint (p)
        };
        let p3 = {
            let p = s.await_immediate ()
            .bind ( map (|()| { println! ("s received"); Result::Ok (()) }))
            .bind ( pause () );
            fixpoint (p)
        };
        identity ()
        .bind (fork (p1))
        .bind (fork (p2))
        .bind (fork (p3))
        .execute (());
    }

}

