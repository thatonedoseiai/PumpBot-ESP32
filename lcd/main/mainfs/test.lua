local l = require("lpb")

bgcol = {16,0,48}
selecting_channels = false

l.set_char_size(12<<6)
on_text = l.draw_text(89, 16, "n", {255,255,255}, bgcol)
off_text = l.draw_text(89, 16, "ff", {255,255,255}, bgcol)
l.set_char_size(18<<6)
on_btn_text = l.draw_text(21, 2, "N", {255,255,255}, bgcol)
off_btn_text = l.draw_text(21, 2, "FF", {255,255,255}, bgcol)
channels_text = {{},{},{},{}}
channels_text_bg = {}
for ck=1,4 do
    channels_text[ck] = l.draw_text(0,32,"CH"..ck.."⤓",{255,255,255},bgcol)
    channels_text_bg[ck] = l.draw_rectangle(129, 187, 62, 21, bgcol)
    l.center_sprites_x(channels_text[ck])
end
k=1
l.set_char_size(42<<6)
spr = {}
f=0
print("lua start")
collectgarbage("stop")
back = l.draw_rectangle(71,62,177,46,bgcol)
back_small = l.draw_rectangle(58,224,51,14,bgcol)
back_on_off = l.draw_rectangle(89,208,12,16,bgcol)
back_btn_text = l.draw_rectangle(21,217,28,21,bgcol)
l.draw_sprites({back,back_small,back_btn_text})
oldrotenc = true
oldleftbtn = true
leftbtncircbuf = {true, true, true, true, true, true, true, true, true, true}
leftbtncircbufi = 1
channel = {0, 0, 0, 0}
channel_active = {true, true, true, true}
xs = {58,111,164,216}
timeout = 0
l.draw_sprites(off_btn_text)

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

print("start loop")
update_screen_text(40, 134, {back}, k, true)

while(true) do
    buttons = l.getgpio()
    if(buttons ~= nil and buttons[1] == 3 and buttons[2] == 1) then
        print("end: "..buttons[1].." "..buttons[2])
        break
    end
    f = l.readrotary()
    if(buttons ~= nil) then
        if(buttons[1] == 0 and buttons[2] == 1) then
            channel_active[k] = not channel_active[k]
            timeout = 500
            if(channel_active[k]) then
                l.set_pwm(k+3, channel[k]*163)
                l.draw_sprites({back_on_off, back_btn_text, on_text[1], off_btn_text[1], off_btn_text[2]})
            else
                l.stop_pwm(k+3)
                l.draw_sprites({back_on_off, back_btn_text, off_text[1], off_text[2], on_btn_text[1]})
            end
        elseif (buttons[1] == 18 and buttons[2] == 1) then
            selecting_channels = not selecting_channels
        end
    end

    if(f ~= nil) then
        if selecting_channels == false then
            if(f[1] == 2) then channel[k] = channel[k] - 1 else channel[k] = channel[k] + 1 end
            if(channel[k] >= 0 and channel[k] <= 100) then
                update_screen_text(40, 134, {back}, k, true)
                l.sprite_move_x({back_small}, xs[k])
                l.set_char_size(12<<6)
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
        else
            if(f[1] == 2) then k = k - 1 else k = k + 1 end
            if k == 5 then
                k = 1
            elseif k == 0 then
                k = 4
            end
            l.draw_sprites({channels_text_bg[k]})
            l.draw_sprites(channels_text[k])
            l.sprite_move_x({back_small}, xs[k])
            l.sprite_move_x({back_on_off}, xs[k]+31)
            l.sprite_move_x(on_text, xs[k]+31)
            l.sprite_move_x(off_text, xs[k]+31)
            update_screen_text(40, 134, {back}, k, true)
            l.set_pwm(k+3, channel[k]*163)
        end
    end
end

delete_sprite(on_text[1])
delete_sprite(off_text[1])
delete_sprite(off_text[2])
delete_sprite(on_btn_text[1])
delete_sprite(off_btn_text[1])
delete_sprite(off_btn_text[2])
delete_sprite(back_small)
delete_sprite(back_on_off)
delete_sprite(back_btn_text)
delete_sprite(back)
