-- OK
A = { i = 1 }

-- OK as this is a function
function foo()
    print("foo")
end
-- OK as it is a reassignment
foo = 1

-- OK
local ok = true

-- NG
bar = { i = 1 }
-- OK as it is a reassignment
bar = 1
-- OK
bar.i = 2

-- NG
kSomeConstant = 1

-- OK
Česká = "czech"
-- NG
večer = 1
