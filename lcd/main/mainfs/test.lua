local l = require("lpb")

k=0
l.set_char_size(42<<6)
spr = {}
f=0
collectgarbage("stop")
print("entering loop!")
back = l.draw_rectangle(71,62,177,46,{16,0,48})
l.draw_sprites({back})
oldf = 0
oldrotenc = true

while(l.getgpio(0)) do
    f = l.readrotary()
    if(f[2] ~= oldf) then
        oldf = f[2]
        if(f[1] == 2) then k = k - 1 else k = k + 1 end
        if(k >= 0 and k <= 100) then
            for s=1,#spr do
                l.delete_sprite(spr[s])
            end
            l.draw_sprites({back})
            spr = l.draw_text(40, 134, string.format("%.0f%%", k), {255,255,255}, {16,0,48});
            l.center_sprites_x(spr)
            l.draw_sprites(spr)
            l.set_pwm(4, k*163)
        else
            if(k > 100) then
                k = 100
            else
                k = 0
            end
        end
    end
end
