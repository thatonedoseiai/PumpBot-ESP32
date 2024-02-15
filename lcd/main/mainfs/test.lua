local l = require("lpb")

k=1
l.set_char_size(42<<6)
spr = {}
f=0
collectgarbage("stop")
print("entering loop!")
back = l.draw_rectangle(71,62,177,46,{16,0,48})
l.draw_sprites({back})
oldf = 0
oldrotenc = true
channel = {0, 0, 0}

while(l.getgpio(0)) do
    f = l.readrotary()
    if(f[2] ~= oldf) then
        oldf = f[2]
        if(f[1] == 2) then channel[k] = channel[k] - 1 else channel[k] = channel[k] + 1 end
        if(channel[k] >= 0 and channel[k] <= 100) then
            for s=1,#spr do
                l.delete_sprite(spr[s])
            end
            l.draw_sprites({back})
            spr = l.draw_text(40, 134, string.format("%.0f%%", channel[k]), {255,255,255}, {16,0,48});
            l.center_sprites_x(spr)
            l.draw_sprites(spr)
            l.set_pwm(k+3, channel[k]*163)
        else
            if(channel[k] > 100) then
                channel[k] = 100
            else
                channel[k] = 0
            end
        end
    end
    if(not l.getgpio(18) and oldrotenc) then
        k = k + 1
        if k == 4 then
            k = 1
        end
    end
    oldrotenc = l.getgpio(18)
end
