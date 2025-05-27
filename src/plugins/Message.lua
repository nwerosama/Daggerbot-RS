function MFPassword(rs_msg)
  if rs_msg.content ~= "!!_farmpw" then
    return
  end

  local passwordText = "Farm password is"
  local pwMapping = {
    [''] = '',
    [''] = ''
  }

  local farm_password = pwMapping[rs_msg.channel_id]
  if farm_password then
    passwordText = string.format("%s `%s`", passwordText, farm_password)
    send_message(rs_msg.channel_id, passwordText)
  end
end
