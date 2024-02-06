local l = require("lpb")

print("hello from lua!")
-- l.set_char_size(14 << 6)
-- k = l.draw_text(100,100,"abcde",{255,255,255},{0,0,0})
-- print("I drew some text for you. Here you go!")
-- print("the sprites are at:")
-- for i=1,#k do
--     print(k[i])
-- end
i=200
k=0
l.set_char_size(48<<6)
spr = {}
print("entering loop!")
while(l.getgpio(0)) do
    if(l.readrotary()[2] ~= k) then
        k = l.readrotary()[2]
        print(k)
        newspr = l.draw_text(40, 176, string.format("%.0f", k), {255,255,255}, {16,0,48});
        for s=1,#spr do
            l.delete_sprite(spr[s])
        end
        l.center_sprites_x(newspr)
        l.draw_sprites(newspr)
        spr = newspr
    end
    if(i == 0) then
        i = 201
        l.wait(100)
    end
    i=i-1
end
