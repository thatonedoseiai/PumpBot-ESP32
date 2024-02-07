local l = require("lpb")

print("hello from lua!")
-- l.set_char_size(14 << 6)
-- k = l.draw_text(100,100,"abcde",{255,255,255},{0,0,0})
-- print("I drew some text for you. Here you go!")
-- print("the sprites are at:")
-- for i=1,#k do
--     print(k[i])
-- end
k=0
l.set_char_size(48<<6)
spr = {}
f=0
collectgarbage("stop")
print("entering loop!")
while(l.getgpio(0)) do
    f = l.readrotary()[2]
    if(f ~= k) then
        k = f
        for s=1,#spr do
            l.delete_sprite(spr[s])
        end
        spr = l.draw_text(40, 176, string.format("%.0f", k), {255,255,255}, {16,0,48});
        l.center_sprites_x(spr)
        l.draw_sprites(spr)
    end
end
