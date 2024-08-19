print("_ENV:\n")
for key, value in pairs(_ENV) do
    -- but not in _G
    local found = false
    for k, v in pairs(_G) do
        if key == k then
            found = true
            break
        end
    end
    if not found then
        print(key)
    end
end

print("globals:")
for key, value in pairs(_G) do
    print(key)
end

print("table:")
for key, value in pairs(table) do
    print(key)
end
