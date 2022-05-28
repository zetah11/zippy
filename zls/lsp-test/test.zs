package Ok

Person := fun (name, age) class
    var name :: String := name
    var age  :: Nat    := age

    rename := fun (var self, to)
        name := to
    
    olden := fun (var self, to)
        age += to
    
    greet := fun (self)
        print("Hello, " + name + "!")
end

-- ok