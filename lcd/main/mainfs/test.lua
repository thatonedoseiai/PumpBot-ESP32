local l = require("lpb")

bgcol = l.background_color();
fgcol = l.foreground_color();
selecting_channels = false

l.set_char_size(12<<6)
wifi_symbol_status = l.wifi_is_connected()
server_symbol_status = l.server_is_connected()
w = l.draw_text(306, 226, "󰖩", fgcol, bgcol)
n = l.draw_text(306, 226, "󰖪", fgcol, bgcol)
sc = l.draw_text(294, 226, "+", fgcol, bgcol)
snc = l.draw_text(294, 226, "-", fgcol, bgcol)
snc = {l.draw_rectangle(294, 226, 12, 12, bgcol)}
if(wifi_symbol_status) then
    l.draw_sprites(w)
else
    l.draw_sprites(n)
end
if(server_symbol_status) then
    l.draw_sprites(sc)
else
    l.draw_sprites(snc)
end
on_text = l.draw_text(85, 16, "n", fgcol, bgcol)
off_text = l.draw_text(85, 16, "ff", fgcol, bgcol)
l.set_char_size(18<<6)
on_btn_text = l.draw_text(19, 2, "N", fgcol, bgcol)
off_btn_text = l.draw_text(19, 2, "FF", fgcol, bgcol)
channels_text = {{},{},{},{}}
channels_text_bg = {}

for ck=1,4 do
    channels_text[ck] = l.draw_text(0,36,"CH"..(ck-1).."⤓",fgcol,bgcol)
    channels_text_bg[ck] = l.draw_rectangle(129, 240-187-21, 62, 21, bgcol)
    l.center_sprites_x(channels_text[ck])
end
f=0
k=1
l.set_char_size(42<<6)
spr = {}
collectgarbage("stop")
back = l.draw_rectangle(71,240-62-46,177,46,bgcol)
back_small = l.draw_rectangle(58,240-224-14,51,14,bgcol)
back_on_off = l.draw_rectangle(89,240-208-16,12,14,bgcol)
back_btn_text = l.draw_rectangle(21,240-217-21,28,21,bgcol)
l.draw_sprites({back,back_btn_text})
-- channel = {0, 0, 0, 0}
xs = {58,111,164,217}
l.draw_sprites(off_btn_text)
l.enable_text_cache_auto_delete(false)

function update_off_text(k)
    if(l.output_off(k)) then
        l.draw_sprites({back_on_off, back_btn_text})
        l.draw_sprites({off_text[1], off_text[2], on_btn_text[1]})
        l.draw_sprites({off_text[2]})
    else
        l.draw_sprites({back_on_off, back_btn_text})
        l.draw_sprites({on_text[1], off_btn_text[1], off_btn_text[2]})
    end
end

function parse_message(message)
    opcode = string.byte(message, 1)
    optable = {
        [1] = function(m)
            v, c, time = select(2, string.unpack(">BI2I1I2", m))
            -- local v = ((string.byte(m, 2) & 0x3f) << 8) | 
            --     (string.byte(m, 3) & 0xff)
            -- local c = string.byte(m, 4) & 0xff
            -- time = ((string.byte(m, 5) & 0x3f) << 8) | 
            --         (string.byte(m, 6) & 0xff)
            if(c < 5 and c > 0) then
                if(time == 0) then
                    l.set_output(c, v)
                else
                    l.set_output_timeout(c, v, time)
                end
            end
        end,
        [3] = function(m)
            local v, c = select(2, string.unpack(">Bi2I1", m)) -- 2nd integer is signed
            -- local v = ((string.byte(m, 2) & 0x3f) << 8) | 
            --     (string.byte(m, 3) & 0xff)
            -- local c = string.byte(m, 4) & 0xff
            if(c < 5 and c > 0) then
                l.increment_output(c, v)
            end
        end,
        [4] = function(m)
            local op, c = string.unpack(">BB", m)
            if(op & 0x80 ~= 0) then
                l.toggle_output(c)
            end
            if(op & 0x40 ~= 0) then
                l.set_output_enable(c, true)
            end
            if(op & 0x20 ~= 0) then
                l.set_output_enable(c, false)
            end
            if(op & 0x10 ~= 0) then
                local s = string.pack(">I2B", l.get_output_value(c), l.output_off(c) and 1 or 0)
                l.server_send_message(s)
            end
            update_off_text(c)
        end,
        [8] = function(m)
            v = select(2, string.unpack(">BI2", m))
            l.disable_hid(v)
        end,
        [10] = function(m)
            v = select(2, string.unpack(">Bi2", m)) -- 2nd integer is signed
            l.increment_hid(v)
        end
    }
    f = optable[opcode & 0xf]
    -- print(f)
    if(f ~= nil) then f(message) end
end

function update_screen_text(x, y, bg, k, center)
    for s=1,#spr do
        l.delete_sprite(spr[s])
    end
    l.draw_sprites(bg)
    spr = l.draw_text(x, y, string.format("%.0f%%", l.get_output_value(k)//163), fgcol, bgcol);
    if(center) then
        l.center_sprites_x(spr)
    end
    l.draw_sprites(spr)
end

update_screen_text(40, 134, {back}, k, true)

while(true) do
    buttons = l.getgpio()
    if(buttons ~= nil and buttons[1] == 3 and buttons[2] == 1) then
        break
    end
    f = l.readrotary()
    if(buttons ~= nil) then
        if(buttons[1] == 0 and buttons[2] == 1) then
            l.toggle_output(k)
            if(l.output_off(k)) then
                l.draw_sprites({back_on_off, back_btn_text})
                l.draw_sprites({off_text[1], off_text[2], on_btn_text[1]})
                l.draw_sprites({off_text[2]})
            else
                l.draw_sprites({back_on_off, back_btn_text})
                l.draw_sprites({on_text[1], off_btn_text[1], off_btn_text[2]})
            end
        elseif (buttons[1] == 18 and buttons[2] == 1) then
            selecting_channels = not selecting_channels
        end
    end

    if(f ~= nil) then
        if selecting_channels == false then
            -- print(f[3])
            if(f[1] == 2) then
                l.increment_output(k, -163 * f[3])
            else
                l.increment_output(k, 163 * f[3])
            end
        else
            if(f[1] == 2) then k = k - f[3] else k = k + f[3] end
            if k >= 5 then
                k = 1
            elseif k <= 0 then
                k = 4
            end
            l.draw_sprites({channels_text_bg[k]})
            l.draw_sprites(channels_text[k])
            l.sprite_move_x({back_small}, xs[k])
            l.sprite_move_x({back_on_off}, xs[k]+27)
            l.sprite_move_x(on_text, xs[k]+27)
            l.sprite_move_x({off_text[1]}, xs[k]+27)
            l.sprite_move_x({off_text[2]}, xs[k]+33)
            update_screen_text(40, 134, {back}, k, true)
        end
    end
    if(l.output_was_updated(k)) then
        update_screen_text(40, 134, {back}, k, true)
        l.sprite_move_x({back_small}, xs[k])
        l.set_char_size(12<<6)
        update_screen_text(xs[k], 2, {back_small}, k, false)
        l.set_char_size(42<<6)
    end
    if(wifi_symbol_status ~= l.wifi_is_connected()) then
        wifi_symbol_status = l.wifi_is_connected()
        if(wifi_symbol_status) then
            l.draw_sprites(w)
        else
            l.draw_sprites(n)
        end
    end
    if(server_symbol_status ~= l.server_is_connected()) then
        print("s: "..server_symbol_status)
        server_symbol_status = l.server_is_connected()
        if(server_symbol_status) then
            l.draw_sprites(sc)
        else
            l.draw_sprites(snc)
        end
    end
    -- print("LUA: GETTING MESSAGE")
    message = l.server_get_message()
    -- print("LUA: FINISHED GETTING MESSAGE")
    if(message ~= nil) then
        parse_message(message)
    end
end

l.enable_text_cache_auto_delete(true);
collectgarbage("collect")
delete_all_sprites()
