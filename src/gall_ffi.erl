-module(gall_ffi).
-export([now/0, read_line/0, write_line/1]).

now() -> os:system_time(second).

%% Read one line from stdin. Returns {ok, Line} or eof.
read_line() ->
    case io:get_line("") of
        eof -> eof;
        {error, _} -> eof;
        Line ->
            Trimmed = string:trim(Line, trailing, "\n\r"),
            {ok, unicode:characters_to_binary(Trimmed)}
    end.

%% Write a line to stdout. MCP is newline-framed.
write_line(Line) ->
    io:put_chars([Line, "\n"]).
