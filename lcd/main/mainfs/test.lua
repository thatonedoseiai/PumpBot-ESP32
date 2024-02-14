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
l.set_char_size(42<<6)
spr = {}
f=0
collectgarbage("stop")
print("entering loop!")
-- i = l.draw_rectangle(60,10,60,80,{255,0,0})
-- l.draw_sprites({i})

-- x = {10, 60, 150, 20, 70, 80, 100, 50}
-- y = {50, 20, 40, 10, 30, 70, 80, 50}
-- height = {90,60,90,70,50,70,80,110}
-- width = {50,100,70,140,120,30,100,80}
-- cols = {{255,0,0},{0,255,0},{255,128,0},{0,0,255},{255,0,100},{128,64,20},{100,0,255},{20,255,255}}
-- sprs = {}
-- for f=1,8 do
--     sprs[2*f-1] = l.draw_rectangle(x[f],y[f],width[f],height[f],cols[f])
--     sprs[2*f] = l.draw_rectangle(x[f],y[f],width[f],height[f],{0x10,0,0x30})
-- end
-- print("finished initing sprites")

-- f = 0
-- while(l.getgpio(0)) do
--     l.draw_sprites({sprs[f+1]})

--     f = (f + 1) % 16
-- end

back = l.draw_rectangle(71,62,177,46,{16,0,48})
l.draw_sprites({back})
x = 3
while(l.getgpio(0)) do
    f = l.readrotary()[2]
    if(f ~= k) then
        k = f
        for s=1,#spr do
            l.delete_sprite(spr[s])
        end
        l.draw_sprites({back})
        spr = l.draw_text(40, 134, string.format("%.0f%%", k), {255,255,255}, {16,0,48});
        l.center_sprites_x(spr)
        -- l.draw_sprites(table.move(spr, x, 3, 1, {}))
        l.draw_sprites(spr)
    end
end
