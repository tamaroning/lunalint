-- OK
for i = 0, 0 do
    print(i)
end

-- OK
for i = 10, 1, -1 do
    print(i)
end

-- OK
-- TODO: should be error?
for i = 10, 1, 2 do
    print(i)
end

-- OK
END = 1
for i = 4, END do
    print(i)
end

-- NG
for i = 10, 1 do
    print(i)
end

-- NG
for i = 99999999999999999, -10 do
    print(i)
end
