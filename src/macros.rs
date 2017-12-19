
#[allow(unused_macros)]

macro_rules! arrow {

    (pause) => (
        pause()
    );

    (pause; $($y:tt)+) => (
        pause().bind(arrow!($($y)+))
    );

    (id) => (
        identity()
    );

    (id; $($y:tt)+) => (
        identity().bind(arrow!($($y)+))
    );

    (val $v:expr) => (
        value($v)
    );

    (val $v:expr; $($y:tt)+) => (
        value($v).bind(arrow!($($y)+))
    );

    ($x:ident => $f:block) => (
        map(|$x| $f)
    );

    ($x:ident => $f:block; $($y:tt)+) => (
        map(|$x| $f).bind(arrow!($($y)+))
    );

    (fix $x:expr) => (
        fixpoint($x)
    );

    (fix $x:expr; $($y:tt)+) => (
        fixpoint($x).bind(arrow!($($y)+))
    );

    (|| $x:expr) => (
        fork($x)
    );

    (|| $x:expr; $($y:tt)+) => (
        fork($x).bind(arrow!($($y)+))
    );

    ($x1:expr , $x2:expr) => (
        product($x1, $x2)
    );

    ($x1:expr , $x2:expr; $($y:tt)+) => (
        product($x1, $x2).bind(arrow!($($y)+))
    );

    ($x:expr) => (
        $x
    );

    ($x:expr; $($y:tt)+) => (
        $x.bind(arrow!($($y)+))
    );

    ($prc:block, $($y:block),*) => (
        product(arrow!($prc), arrow!($($y),*))
    );
}

