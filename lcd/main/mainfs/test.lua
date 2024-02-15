local l = require("lpb")

k=1
l.set_char_size(42<<6)
spr = {}
bgcol = {16,0,48}
f=0
collectgarbage("stop")
print("entering loop!")
back = l.draw_rectangle(71,62,177,46,bgcol)
back_small = l.draw_rectangle(58,223,51,15,bgcol)
l.draw_sprites({back,back_small})
oldf = 0
oldrotenc = true
channel = {0, 0, 0, 0}
xs = {58,111,164,216}

function update_screen_text(x, y, bg, k, center)
    for s=1,#spr do
        l.delete_sprite(spr[s])
    end
    l.draw_sprites(bg)
    spr = l.draw_text(x, y, string.format("%.0f%%", channel[k]), {255,255,255}, {16,0,48});
    if(center) then
        l.center_sprites_x(spr)
    end
    l.draw_sprites(spr)
end

update_screen_text(40, 134, {back}, k, true)
while(l.getgpio(0)) do
    f = l.readrotary()
    if(f[2] ~= oldf) then
        oldf = f[2]
        if(f[1] == 2) then channel[k] = channel[k] - 1 else channel[k] = channel[k] + 1 end
        if(channel[k] >= 0 and channel[k] <= 100) then
            update_screen_text(40, 134, {back}, k, true)
            l.set_char_size(12<<6)
            l.sprite_move_x({back_small}, xs[k])
            update_screen_text(xs[k], 2, {back_small}, k, false)
            l.set_char_size(42<<6)
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
        if k == 5 then
            k = 1
        end
        update_screen_text(40, 134, {back}, k, true)
        l.set_pwm(k+3, channel[k]*163)
    end
    oldrotenc = l.getgpio(18)
end
