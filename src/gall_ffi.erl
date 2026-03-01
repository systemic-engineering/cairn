-module(gall_ffi).
-export([now/0]).
now() -> os:system_time(second).
