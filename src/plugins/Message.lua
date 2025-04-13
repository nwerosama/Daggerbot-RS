function MFPassword(rs_msg)
  if rs_msg.content ~= "!!_farmpw" then
    return
  end

  local passwordText = "Farm password is"
  local pwMapping = {
    ['1266224299174396045'] = 'hickory',
    ['1266224585007824986'] = 'hillcreek'
  }

  local farm_password = pwMapping[rs_msg.channel_id]
  if farm_password then
    passwordText = string.format("%s `%s`", passwordText, farm_password)
    send_message(rs_msg.channel_id, passwordText)
  end
end
