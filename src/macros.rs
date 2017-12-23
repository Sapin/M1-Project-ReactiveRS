
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

    (ret $v:expr) => (
        value($v)
    );

    (ret $v:expr; $($y:tt)+) => (
        value($v).bind(arrow!($($y)+))
    );

    ($x:ident => $f:block) => (
        map(|$x| $f)
    );

    ($x:pat => $f:block) => (
        map(|$x| $f)
    );

    ($x:ident => $f:block; $($y:tt)+) => (
        map(|$x| $f).bind(arrow!($($y)+))
    );

    ($x:pat => $f:block; $($y:tt)+) => (
        map(|$x| $f).bind(arrow!($($y)+))
    );

    (mv $x:ident => $f:block) => (
        map(move |$x| $f)
    );

    (mv $x:pat => $f:block) => (
        map(move |$x| $f)
    );

    (mv $x:ident => $f:block; $($y:tt)+) => (
        map(move |$x| $f).bind(arrow!($($y)+))
    );

    (mv $x:pat => $f:block; $($y:tt)+) => (
        map(move |$x| $f).bind(arrow!($($y)+))
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

    (emit $x:expr) => (
        $x.emit()
    );

    (emit $x:expr; $($y:tt)+) => (
        $x.emit().bind(arrow!($($y)+))
    );

    (emit $x:expr, $v:expr) => (
        value($v).bind($x.emit())
    );

    (emit $x:expr, $v:expr; $($y:tt)+) => (
        value($v).bind($x.emit()).bind(arrow!($($y)+))
    );

    (present $s:expr, $then:expr, $else:expr) => (
        $s.present($then, $else)
    );

    (present $s:expr, $then:expr, $else:expr; $($y:tt)+) => (
        $s.present($then, $else).bind(arrow!($($y)+))
    );

    (await immediate $s:expr) => (
        $s.await_immediate()
    );

    (await immediate $s:expr; $($y:tt)+) => (
        $s.await_immediate().bind(arrow!($($y)+))
    );

    (await $s:expr) => (
        $s.await()
    );

    (await $s:expr; $($y:tt)+) => (
        $s.await().bind(arrow!($($y)+))
    );

    ($b:block) => (
        map(|_| $b)
    );

    ($b:block; $($y:tt)+) => (
        map(|_| $b).bind(arrow!($($y)+))
    );

    ($x:expr) => (
        $x
    );

    ($x:expr; $($y:tt)+) => (
        $x.bind(arrow!($($y)+))
    );

}

