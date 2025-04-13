Response = {}

-- Prefixes are purely optional and can be omitted
local popular_prefixes = {
  'good', 'great'
}
local popular_suffixes = {
  'all', 'everyone', 'lads',
  'guys', 'everybody', 'yall',
  'y\'all', 'my neighbors', 'my neighbours',
  'daggerbot', 'daggerbots'
}

Response.incomingArrays = {
  morning = {
    prefix = popular_prefixes,
    suffix = popular_suffixes
  },
  afternoon = {
    prefix = popular_prefixes,
    suffix = popular_suffixes
  },
  evening = {
    prefix = popular_prefixes,
    suffix = popular_suffixes
  },
  night = {
    prefix = popular_prefixes,
    suffix = popular_suffixes
  }
}

local function escape(s)
  return (s:gsub('([^%w])', '%%%1'))
end

local function getDoty()
  local day = tonumber(os.date('%j'))
  local last_digit = day % 10
  local suffixes = {'st', 'nd', 'rd'}
  local suffix = 'th'

  if day < 11 or day > 13 then
    suffix = suffixes[last_digit] or suffix
  end

  return string.format('%d%s', day, suffix)
end

function Response.outgoingArrays(rs_msg, keyword)
  local PersonnyMcPerson = string.format('**%s**', rs_msg.author.global)
  local arrays = {
    morning = {
      '### Have a wonderful day ahead of you!',
      '# *DID YOU HAVE A GREAT SLEEP LAST NIGHT?*',
      'Gooooood morning to you!',
      'Howdy! How\'s your morning?',
      'Time to get started with today\'s stuff!',
      'Enjoy the breakfast and start your important day!',
      'Nuh! No morning message for you!\n*Just kidding, good morning!!*',
      'https://tenor.com/view/skyrim-intro-awake-finally-awake-gif-22633549',
      'https://tenor.com/view/rambo-family-rambo-rise-and-shine-wake-up-gif-22012440',
      'https://tenor.com/view/good-morning-vietnam-robin-williams-classic-announcer-radio-gif-4844905',
      'Good morning to you! You know what else is toastally awesome?\nOur sponsor: breakfast! Let’s get this bread—rise and dine with some eggcellent breakfast!',
      'Good morning to you! But you know what else is good?\nOur sponsor: breakfast! That\'s right, folks—get started with getting out of bed and grabbing some breakfast! Trust me, it\'s the ultimate way to kickstart your day. Don\'t miss out!',
      'Is it Friday yet? This week is getting boring already!!',
      'Good morning! Have a cookie to start your day with. :cookie:',
      string.format('You have reached %s day of the year, also good morning to you as well!', getDoty()),
      string.format('Gm %s', PersonnyMcPerson),
      string.format('## Morning %s!', PersonnyMcPerson),
      string.format('Good morning %s!', PersonnyMcPerson),
      string.format('Rise and shine, %s!', PersonnyMcPerson),
      string.format("A new day, a new start, %s!", PersonnyMcPerson),
      string.format('Morning %s, did you sleep great?', PersonnyMcPerson),
      string.format('Hope you enjoyed your breakfast, %s!', PersonnyMcPerson),
      string.format('Don\'t forget to do your morning routine, %s!', PersonnyMcPerson),
      string.format('*Uhh...* What time is it? Oh right, morning %s..', PersonnyMcPerson),
      string.format('Morning and hope you had a good dream last night, %s!', PersonnyMcPerson),
      string.format('Here, have some pancakes for breakfast, %s! :pancakes:', PersonnyMcPerson),
      string.format('Rise and shine, sleepyhead %s! Ready to start your day?', PersonnyMcPerson),
      string.format('Oh good grief, is it Monday already?? Anyways, morning %s..', PersonnyMcPerson),
      string.format('*opens blinds wide enough to blast sunrays into the room*\nWakey wakey, %s. Time to get up!', PersonnyMcPerson),
      string.format('Wake up and smell the delicious pancakes, %s! It\'s a brand new day with many possibilities!', PersonnyMcPerson),
      string.format('This time I can now shout! So here we go! 1..2..3\n*inhales*\n# MORNING %s!', PersonnyMcPerson.upper(PersonnyMcPerson))
    },
    afternoon = {
      '### Quite a wonderful weather today!',
      'Hope you had a good day so far',
      'Weather doesn\'t look too bad outside right?',
      'Did you have a wonderful and productive day so far?',
      'Afternoon already? Jeez, time go brrrr!',
      'We\'re halfway through the day, aren\'t we?',
      'Are we not supposed to be at work or something? Oh well, good afternoon regardless!',
      'I hope I won\'t let you down with this very delicious cupcake! :cupcake:',
      string.format('Afternoon %s!', PersonnyMcPerson),
      string.format('Good afternoon %s!', PersonnyMcPerson),
      string.format('What a nice day to spend quality time outside, %s!', PersonnyMcPerson),
      string.format('Did you enjoy your day yet, %s?', PersonnyMcPerson),
      string.format('How\'s the trip outside, %s?', PersonnyMcPerson),
      string.format('~~Morning~~ Afternoon %s!', PersonnyMcPerson),
      string.format('Ready to enjoy your delicious lunch, %s?', PersonnyMcPerson),
      string.format('Hi there, adventurer %s! What\'s on your agenda for rest of the day?', PersonnyMcPerson),
      string.format('Afternoon %s, back from your trip outside?', PersonnyMcPerson),
      string.format('How are you doing today, %s?', PersonnyMcPerson),
      string.format('Good afternoon %s, I hope you\'re having a more fanastic day than that poor particular penguin in Antarctica that slipped!', PersonnyMcPerson),
      string.format('Good afternoon %s! Hope your day is going better than a penguin in a snowstorm in Antarctica!', PersonnyMcPerson),
      string.format('Afternoon %s! What\'s the current progress on your todo list? Did you finish them?', PersonnyMcPerson),
      string.format('Afternoon %s! How\'s the quest for the perfect snack coming along?\nRemember, it\'s all about the journey, not the destination... *unless the destination is the... fridge.*', PersonnyMcPerson)
    },
    evening = {
      'May I suggest sleep?',
      'I can\'t believe the time flies so quickly!',
      'Today is almost over, you deserve some rest!',
      'You look tired, ready to go to sleep yet?',
      'Being outside was an exhausting experience, wasn\'t it?',
      'Did you have a good day so far?',
      'So, what\'s for dinner?',
      string.format('Evening %s!', PersonnyMcPerson),
      string.format('Hope you enjoyed your dinner, %s!', PersonnyMcPerson),
      string.format('Good evening %s!', PersonnyMcPerson),
      string.format('You heard me! %s, it\'s almost dinner time!', PersonnyMcPerson),
      string.format('How\'s your day going, %s?', PersonnyMcPerson),
      string.format('%s, may I suggest... *sleep?*', PersonnyMcPerson),
      string.format('Good evening %s! Just remember, the absolute best part of the day is yet to come... **bedtime!** Who\'s with me? <a:MichaelSurprised:1016297232263286825>', PersonnyMcPerson)
    },
    night = {
      'Nighty night!',
      'Finally, the day is over. Rest well!',
      'Another day done, now take some rest',
      'Alrighty mighty, have a good sleep and see you tomorrow!',
      string.format('Gn %s', PersonnyMcPerson),
      string.format('Good night %s!', PersonnyMcPerson),
      string.format('Night %s!', PersonnyMcPerson),
      string.format('Sweet dreams, %s', PersonnyMcPerson),
      string.format('Don\'t fall out of sky in your dreamworld, %s!', PersonnyMcPerson),
      string.format('I hope tomorrow is a good day for you, %s!', PersonnyMcPerson),
      string.format('Have a good sleep, %s!', PersonnyMcPerson),
      string.format('I :b:et you a cookie if you actually slept through the night! %s', PersonnyMcPerson),
      string.format('Sleep well %s!', PersonnyMcPerson),
      string.format('Close your eyelids and sleep, %s', PersonnyMcPerson),
      string.format('Good night %s and hope your pillow is nice and cold!', PersonnyMcPerson),
      string.format('# Night %s!', PersonnyMcPerson),
      string.format('You should try maintaining your sleep schedule if you\'re really that tired, %s', PersonnyMcPerson),
      string.format('Goodnight %s, time to recharge your social batteries for tomorrow!', PersonnyMcPerson),
      string.format('Have a good night %s, don\'t let the bed bugs bite!', PersonnyMcPerson),
      string.format('Sweet dreams, %s! Hope your dreams are as wild as unpredictable as a Netflix algorithm!', PersonnyMcPerson)
    }
  }

  return arrays[keyword]
end

function Response.respond(rs_msg, keyword)
  local outgoingArray = Response.outgoingArrays(rs_msg, keyword)
  local randomIndex = math.random(#outgoingArray)
  local respMessage = outgoingArray[randomIndex]

  local incomingArray = Response.incomingArrays[keyword]
  local parray = table.concat(incomingArray.prefix, "|")
  local sarray = table.concat(incomingArray.suffix, "|"):gsub("%w+", escape)
  local pattern = string.format('^(%s)?\\s?%s\\s+(%s)\\b', parray, keyword, sarray)

  if rs_regex(pattern, string.lower(rs_msg.content)) then
    send_message(rs_msg.channel_id, respMessage)
  end
end

return Response
