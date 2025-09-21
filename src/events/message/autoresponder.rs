pub const PREFIXES: &[&str] = &["good", "great"];
pub const SUFFIXES: &[&str] = &["all", "everyone", "everybody", "lads", "guys", "yall", "y\'all", "homies", "chat"];

pub enum Keywords {
  Morning,
  Afternoon,
  Evening,
  Night
}

impl std::fmt::Display for Keywords {
  fn fmt(
    &self,
    f: &mut std::fmt::Formatter<'_>
  ) -> std::fmt::Result {
    let kw = match self {
      Self::Morning => "morning",
      Self::Afternoon => "afternoon",
      Self::Evening => "evening",
      Self::Night => "night"
    };
    write!(f, "{kw}")
  }
}

pub fn match_keywords(
  kw: Keywords,
  author: String
) -> Vec<String> {
  let author = format!("**{author}**");
  match kw {
    Keywords::Morning => vec![
      "Gooooood morning to you!".to_string(),
      "Howdy! How's your morning?".to_string(),
      "### Have a wonderful day ahead of you!".to_string(),
      "Time to get started with today's stuff!".to_string(),
      "# *DID YOU HAVE A GREAT SLEEP LAST NIGHT?*".to_string(),
      "Enjoy the breakfast and start your important day!".to_string(),
      "Is it Friday yet? This week is getting boring already!!".to_string(),
      "Good morning! Have a cookie to start your day with! :cookie:".to_string(),
      "Nuh! No morning message for you!\n*Just kidding, good morning!!!*".to_string(),
      "https://tenor.com/view/skyrim-intro-awake-finally-awake-gif-22633549".to_string(),
      "https://tenor.com/view/rambo-family-rambo-rise-and-shine-wake-up-gif-22012440".to_string(),
      "https://tenor.com/view/good-morning-vietnam-robin-williams-classic-announcer-radio-gif-4844905".to_string(),
      "Good morning to you! You know what else is toastally awesome?\nOur sponsor: breakfast! Let's get this bread—rise and dine with some \
       eggcellent breakfast!"
        .to_string(),
      "Good morning to you! But you know what else is good?\nOur sponsor: breakfast! That's right, folks—get started with getting out of bed and \
       grabbing some breakfast! Trust me, it's the ultimate way to kickstart your day. Don\'t miss out!"
        .to_string(),
      format!("Gm {author}"),
      format!("## Morning {author}!"),
      format!("Good morning {author}!"),
      format!("Rise and shine, {author}!"),
      format!("A new day, a new start, {author}!"),
      format!("Morning {author}, did you sleep great?"),
      format!("Hope you enjoyed your breakfast, {author}!"),
      format!("Don\'t forget to do your morning routine, {author}!"),
      format!("*Uhh...* What time is it? Oh right, morning {author}.."),
      format!("Morning and hope you had a good dream last night, {author}!"),
      format!("Here, have some pancakes for breakfast, {author}! :pancakes:"),
      format!("Rise and shine, sleepyhead {author}! Ready to start your day?"),
      format!("Oh good grief, is it Monday already?? Anyways, morning {author}.."),
      format!("Morning has broken and so has {author}'s alarm clock! Rise and shine anyway!"),
      format!("This time I can now shout! So here we go! 1..2..3\n*inhales*\n# MORNING {author}!"),
      format!("The early bird gets the worm but {author} gets a friendly morning greeting instead!"),
      format!(
        "You have reached {} day of the year, also good morning to you as well!",
        asahi::utils::get_doty()
      ),
      format!("*opens blinds wide enough to blast sunrays into the room*\nWakey wakey, {author}. Time to get up!"),
      format!("Wake up and smell the delicious pancakes, {author}! It's a brand new day with many possibilities!"),
    ],
    Keywords::Afternoon => vec![
      "Hope you had a good day so far".to_string(),
      "### Quite a wonderful weather today!".to_string(),
      "Afternoon already? Jeez, time go brrrr!".to_string(),
      "We\'re halfway through the day, aren\'t we?".to_string(),
      "Weather doesn\'t look too bad outside right?".to_string(),
      "Did you have a wonderful and productive day so far?".to_string(),
      "I hope I won\'t let you down with this very delicious cupcake! :cupcake:".to_string(),
      "Are we not supposed to be at work or something? Oh well, good afternoon regardless!".to_string(),
      format!("Afternoon {author}!"),
      format!("Good afternoon {author}!"),
      format!("~~Morning~~ Afternoon {author}!"),
      format!("How's the trip outside, {author}?"),
      format!("How are you doing today, {author}?"),
      format!("Did you enjoy your day yet, {author}?"),
      format!("Ready to enjoy your delicious lunch, {author}?"),
      format!("Afternoon {author}, back from your trip outside?"),
      format!("What a nice day to spend some quality time outside, {author}!"),
      format!("{author}'s afternoon agenda:\n1. Read this message\n2. ???\n3. Profit!"),
      format!("Hi there, adventurer {author}!\nWhat's on your agenda for the rest of the day?"),
      format!("Afternoon {author}! What's the current progress on your todo list so far? Did you finish them?"),
      format!("Good afternoon {author}! Hope your day is going better than a penguin in a snowstorm in Antarctica!"),
      format!("Afternoon {author}! Fun fact; This is the perfect time to start procrastinating on your evening plans!"),
      format!("Good afternoon {author}, I hope you're having a more fanastic day than that poor particular penguin in Antarctica that slipped!"),
      format!(
        "Afternoon {author}! How's the quest for the perfect snack coming along?\nRemember, it's all about the journey, not the destination... \
         *unless the destination is the... fridge.*"
      ),
    ],
    Keywords::Evening => vec![
      "May I suggest sleep?".to_string(),
      "So, what's for dinner?".to_string(),
      "Did you have a good day so far?".to_string(),
      "What are you having for tonight's tea?".to_string(),
      "What's for dinner? Wrong answers only!".to_string(),
      "You look tired, ready to go to sleep yet?".to_string(),
      "I can't believe the time flies so quickly!".to_string(),
      "Today is almost over, you deserve some rest!".to_string(),
      "Being outside was an exhausting experience, wasn't it?".to_string(),
      format!("Evening {author}!"),
      format!("Good evening {author}!"),
      format!("{author}, may I suggest... *sleep?*"),
      format!("How's your day going so far, {author}?"),
      format!("Hope you enjoyed your dinner, {author}!"),
      format!("Hope you enjoyed your afternoon tea, {author}!"),
      format!("You heard me! {author}, it's almost dinner time!"),
      format!(
        "Good evening {author}! Just remember, the absolute best part of the day is yet to come... **bedtime!** Who's with me?? \
         <a:MichaelSurprised:1016297232263286825>"
      ),
    ],
    Keywords::Night => vec![
      "Night!".to_string(),
      "Nighty night!".to_string(),
      "Finally, the day is over! Rest well!".to_string(),
      "Another day done, now take some well-deserved rest!".to_string(),
      "Alrighty mighty, have a good sleep and see you tomorrow!!".to_string(),
      format!("Gn {author}!"),
      format!("Night {author}!"),
      format!("# Night {author}!"),
      format!("Sweet dreams, {author}!"),
      format!("Have a good sleep, {author}!"),
      format!("Close your eyes and sleep, {author}"),
      format!("Good night and sleep well, {author}!"),
      format!("I hope tomorrow is a good day for you, {author}!"),
      format!("Don't fall out of sky in your dreamworld, {author}!"),
      format!("Good night {author} and may your pillow be forever cold!"),
      format!("Have a good night {author}, don't let the bed bugs bite!"),
      format!("Goodnight {author}, time to recharge your social batteries for tomorrow!"),
      format!("I'll give you a :cookie: if you actually slept through the night, {author}!"),
      format!("Sweet dreams, {author}! May your dreams be as wild as the YouTube algorithm!"),
      format!("Good night {author}! Remember that sleep is just a timetravel to the breakfast!"),
      format!("You should try maintaining your sleep schedule if you're really that tired, {author}!"),
    ]
  }
}
