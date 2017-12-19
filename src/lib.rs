#![type_length_limit="2097152"]

pub mod runtime;
pub mod arrow;
pub mod signal;
#[macro_use]
pub mod macros;

#[cfg(test)]
mod tests {

    use std::result::{Result};

    use arrow::{Arrow};
    use arrow::prim::{identity,value,map,pause,fixpoint,product,fork};

    use signal::{Signal};
    use signal::prim::{PureSignal,ValueSignal,UniqSignal};

    //#[test]
    fn test_pause () {
        product (
            map (|()| { println!("foo"); })
            .bind (pause ())
            .bind (map (|()| { println!("bar"); }))
        ,   map (|()| { println!("foo"); })
            .bind (pause ())
            .bind (map (|()| { println!("bar"); })) 
        )
        .execute_seq (((),()));
    }

    //#[test]
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
        .execute_seq (());
    }

    //#[test]
    fn test_value_signal () {
        let s = ValueSignal::new (Box::new (|a: u32, b: u32| -> u32 {a+b}));
        let p1 = identity ()
        .bind  ( value::<(),u32> (32) )
        .bind  ( s.emit () )
        .bind  ( value::<(),u32> (10) )
        .bind  ( s.emit () )
        .bind  ( pause () )
        .bind  ( value::<(),u32> (42) )
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
        let p4 = {
            let p = s.await ()
            .bind ( map (|i: u32| { println! ("{}", i); Result::Ok (()) }));
            fixpoint (p)
        };
        identity ()
        .bind (fork (p1))
        .bind (fork (p2))
        .bind (fork (p3))
        .bind (fork (p4))
        .execute_seq (());
    }

    #[test]
    fn test_uniq_signal () {
        let (s,await) = UniqSignal::new (Box::new (|a: u32, b: u32| -> u32 {a+b}));
        let p1 = identity ()
        .bind  ( value::<(),u32> (32) )
        .bind  ( s.emit () )
        .bind  ( value::<(),u32> (10) )
        .bind  ( s.emit () )
        .bind  ( pause () )
        .bind  ( value::<(),u32> (42) )
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
        let p4 = {
            let p = await
            .bind ( map (|i: u32| { println! ("{}", i); Result::Ok (()) }));
            fixpoint (p)
        };

        identity ()
        .bind (fork (p1))
        .bind (fork (p2))
        .bind (fork (p3))
        .bind (fork (p4))
        .execute_par (4,());
    }

    #[test]
    fn test_macro_1 () {
        let p = arrow!(i => { let (a,b) : (u32,u32) = i; println!("({}, {})\n", a, b); });
        arrow!(
            val (5u32, 41u32);
            pause;
            pause;
            arrow!(fix arrow!(
                i => {
                    if i < 10 {
                        println!("{}", i); Ok(i + 1)
                    } else { 
                        Result::Err(i)
                    }
                }
            )), arrow!(i => { i + 1 });
            || p;
            id
        ).execute_seq(());
    }

}

