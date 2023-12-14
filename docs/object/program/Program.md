# Representation of OpenGL Program Object

Program object by default is in some state -- default?
Program encompasses multiple shader stages.
It can have multiple shaders for the same stage attached to itself
as well as one shader can be attached to multiple programs

Each Stage has an interface. In order for program to be correct there must more or less match.
One exception that comes to mind is using constant attribute input.
There are rules that govern if two interfaces match
Initially I will consider only matching by using the location specifier since it can
be encoded in type easily with tuples.
Match by parameter name will be difficult to encode in type system, compile time check maybe?
Similarly parameter qualification may be painful and realllly verbose but perhaps default
type parameters will do the trick -- I need to delve into GLSL spec a bit more.

Programs have associated lists of resources that they use.
These lists seem to be good starting point for modelling the type.
There are multiple program interfaces, here are some more notable ones:
- UNIFORM corresponds to the set of active uniform variables used by program.
- UNIFORM_BLOCK corresponds to the set of active uniform blocks used by program.
- ATOMIC_COUNTER_BUFFER corresponds to the set of active atomic counter buffer binding points used by program.
- PROGRAM_INPUT corresponds to the set of active input variables used by the
first shader stage of program. If program includes multiple shader stages,
input variables from any shader stage other than the first will not be enumerated.
- PROGRAM_OUTPUT corresponds to the set of active output variables used by the
last shader stage of program. If program includes multiple shader stages,
output variables from any shader stage other than the last will not be enumerated.
- BUFFER_VARIABLE corresponds to the set of active buffer variables used by program
- SHADER_STORAGE_BLOCK corresponds to the set of active shader storage blocks used by program

This represents an ownership model of sorts though things might be different
when using separable programs.