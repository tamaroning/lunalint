function example_function(x)
    if x > 10 then
        print("x is greater than 10")
    elseif x < 0 then
        print("x is less than 0")
    else
        print("x is between 0 and 10")
        if x == 5 then
            return "x is exactly 5"
            print("This will never print") -- Unreachable code
        else
            print("This is another else block")
        end
    end
end

print(example_function(5))
