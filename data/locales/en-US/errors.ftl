# Lexing and parsing errors
E1000 = Encountered unexpected token during parsing. Found: {$actual}, Expected: {$expected}
E1001 = Encountered unknown escape sequence '{$sequence}'.
E1002 = Unterminated string encountered.
E1003 = Expected valid expression. Found invalid token: {$input}
E1004 = Expected valid expression. Found reserved keyword: {$input}

# Compiler errors
E2000 = Internal Compiler Error (please report this as a bug with code to reproduce the issue).

E2100 = The maximum number of upvalues (256) has been exceeded for this compilation unit.
E2101 = The maximum number of constants (256) has been exceeded for this compilation unit.

E2200 = The new method cannot specify a return type.
E2201 = The new method must specify self as the first parameter.
E2202 = Class operators must specify self as the first parameter.
E2203 = The new method must call super first when deriving from another class.
E2204 = The new method cannot return a value.
E2205 = The 'super' keyword can only be used inside of methods of classes.
E2206 = The self parameter of methods cannot specify a type.
E2207 = Function parameters must have unique names. (TODO: Include duplicate names)

E2300 = The class {$name} is already defined in this scope.
E2301 = The function {$name} is already defined in this scope.

E2500 = The 'return' keyword can only be used inside of functions or methods.
E2501 = The 'break' keyword can only be used inside of loops.
E2502 = The 'continue' keyword can only be used inside of loops.
E2503 = The error propagate operator '!!' can only be used inside of functions or methods.
E2504 = Invalid usage of the 'export' keyword.

# Runtime errors
E3000 = The value cannot be converted to a boolean.
E3001 = The value cannot be converted to an int.
E3002 = The value cannot be converted fo a float.
E3003 = The value cannot be converted to an array.
E3004 = The value cannot be converted to a string.
E3005 = The value cannot be converted to a symbol.
E3006 = The value cannot be converted to an object.
E3007 = The value cannot be converted to a class.

E3100 = Type assertion failed due to mismatching types.
E3101 = Type assertion failed due to unexpected null value.
E3102 = Type assertion failed due to the target value not being a boolean type.
E3103 = Type assertion failed due to the target value not being a numeric type.
E3104 = Type assertion failed due to the target value not being a function type.
E3105 = Type assertion failed due to encountering invalid super type.

E3200 = Attempted to divide by zero.

E3300 = Global variable {$name} cannot be redefined.
E3301 = Global variable {$name} could not be found.
E3302 = Global operator {$name} could not be found.

# System errors
E4000 = A panic has occurred. {$message}
E4001 = IO error occurred. {$message}
