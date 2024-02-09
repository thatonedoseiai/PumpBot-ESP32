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

back = l.draw_rectangle(62,10,42,60,{16,0,48})
back2 = l.draw_rectangle(104,10,42,60,{16,0,48})
back3 = l.draw_rectangle(146,10,47,60,{16,0,48})
l.draw_sprites({back, back2, back3})
x = 3
while(l.getgpio(0)) do
    f = l.readrotary()[2]
    if(f ~= k) then
        if k%100 == 99 then
            x = 1
        elseif k%10 == 9 then
            x = 2
        else
            x = 3
        end
        k = f
        for s=1,#spr do
            l.delete_sprite(spr[s])
        end
        l.draw_sprites(table.move({back, back2, back3}, x, 3, 1, {}))
        spr = l.draw_text(40, 176, string.format("%03.0f%%", k), {255,255,255}, {16,0,48});
        l.sprite_set_draw({spr[#spr]}, false)
        if x == 3 then
            l.sprite_set_draw({spr[1],spr[2]}, false)
        elseif x == 2 then
            l.sprite_set_draw({spr[1]}, false)
        end
        l.center_sprites_x(spr)
        -- l.draw_sprites(table.move(spr, x, 3, 1, {}))
        l.draw_sprites(spr)
    end
end
