local l = require("lpb")

print("hello from lua!")
l.set_char_size(14 << 6)
k = l.draw_text(100,100,"abcde",{255,255,255},{0,0,0})
print("I drew some text for you. Here you go!")
print("the sprites are at:")
for i=1,#k do
    print(k[i])
end
i=200
while(l.getgpio(0)) do
    print(l.readrotary()[2])
    if(i == 0) then
        i = 201
        l.wait(10)
    end
    i=i-1
end
