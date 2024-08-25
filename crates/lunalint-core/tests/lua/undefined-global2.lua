function f2()
    -- OK
    print(G2)
end

function f()
    G2 = 1
end

f()
f2()

-- OK
print(G2)
