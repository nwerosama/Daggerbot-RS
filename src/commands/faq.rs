use {
  crate::{
    BotResult,
    internals::config::BINARY_PROPERTIES
  },
  asahi::{
    canvas::{
      Canvas,
      ImageFormat,
      templates::explorer::{
        Metadata,
        Style,
        Theme,
        file_explorer
      }
    },
    utils::ansi
  },
  poise::{
    ChoiceParameter,
    CreateReply,
    serenity_prelude::{
      CreateAttachment,
      CreateEmbed
    }
  }
};

static FARMSIM_DOCS: &str = "Documents\\My Games\\FarmingSimulator2025";

#[allow(clippy::enum_variant_names)] // This warning is so stupid...
#[derive(Debug, ChoiceParameter)]
pub enum Questions {
  #[name = "[FS] Delete shader_cache folder"]
  FsDeleteShaderCacheFolder,
  #[name = "[FS] Logfile location"]
  FsLogFileLocation,
  #[name = "[FS] Verifying game files"]
  FsVerifyGameFiles,
  #[name = "[FS] Enabling the console"]
  FsEnableDevConsole
}

/// List of popular answered questions
#[poise::command(slash_command)]
pub async fn faq(
  ctx: super::PoiseContext<'_>,
  #[description = "The question you want answers for"] question: Questions
) -> BotResult {
  match question {
    Questions::FsDeleteShaderCacheFolder => {
      let filename = "shader_cache.jpg";
      let img = farmsim_docs(9).to_bytes(Some(ImageFormat::Jpeg { quality: 100 }))?;

      let description = [
        "If your game keeps crashing shortly after opening your game, then the shaders might be an issue.",
        &format!("To resolve this, you can go to `{FARMSIM_DOCS}` and delete the folder called `shader_cache`")
      ]
      .join("\n");

      let embed = build_faq("Deleting your shader_cache folder", description, Some(filename));
      ctx
        .send(CreateReply::default().embed(embed).attachment(CreateAttachment::bytes(img, filename)))
        .await?;
    },
    Questions::FsLogFileLocation => {
      let filename = "log_file.jpg";
      let img = farmsim_docs(21).to_bytes(Some(ImageFormat::Jpeg { quality: 100 }))?;

      let embed = build_faq(
        "Finding your log file",
        format!("Your game's log file is located in `{FARMSIM_DOCS}` and upload it to Discord channel so other people can help solve your issue!"),
        Some(filename)
      );
      ctx
        .send(CreateReply::default().embed(embed).attachment(CreateAttachment::bytes(img, filename)))
        .await?;
    },
    Questions::FsVerifyGameFiles => {
      let image = include_bytes!("../../assets/faq/verify-gamefiles.png");
      let filename = "verify-gamefiles.png";

      let steam_panel = [
        &format!("{} (Top)", ansi::Blue::BOLD.paint("Steam")),
        "1. Go to your library and right-click on Farming Simulator 25",
        "2. Click on 'Properties' and navigate to 'Installed Files'",
        "3. Click on 'Verify integrity of game files'",
        "4. Steam will scan your game installation directory and will redownload anything that is deemed corrupted or tampered with"
      ]
      .join("\n");

      let egs_panel = [
        &format!("{} (Bottom)", ansi::Black::BOLD.paint("Epic Games")),
        "1. Go to your library and click on 3 dots (...)",
        "2. Click on 'Manage' and then click on 'Verify'",
        "3. Epic Launcher will scan your game installation directory and will redownload anything that is deemed corrupted or tampered with"
      ]
      .join("\n");

      let description = [
        "You can verify your game files if you had experienced any issues with your game:```ansi\n",
        &steam_panel,
        "\n",
        &egs_panel,
        "\n```"
      ]
      .join("\n");

      let embed = build_faq("Verifying your game files", description, Some(filename));
      ctx
        .send(
          CreateReply::default()
            .embed(embed)
            .attachment(CreateAttachment::bytes(&image[..], filename))
        )
        .await?;
    },
    Questions::FsEnableDevConsole => {
      let image = include_bytes!("../../assets/faq/enable-console.png");
      let filename = "enable-console.png";

      let embed = build_faq(
        "Enabling the developer console",
        format!(
          "Head over to `game.xml` in `{FARMSIM_DOCS}` and find the section that mentions `<controls>false</controls>` inside development section, \
           change it to `true`, now fire up the game and confirm that the console is enabled by pressing the backtick button!"
        ),
        Some(filename)
      );
      ctx
        .send(
          CreateReply::default()
            .embed(embed)
            .attachment(CreateAttachment::bytes(&image[..], filename))
        )
        .await?;
    }
  }

  Ok(())
}

fn build_faq(
  title: &'static str,
  description: String,
  image: Option<&'static str>
) -> CreateEmbed<'static> {
  let mut embed = CreateEmbed::default()
    .color(BINARY_PROPERTIES.embed_colors.primary())
    .title(title)
    .description(description);

  if image.is_some() {
    embed = embed.image(format!("attachment://{}", image.unwrap()));
  }

  embed
}

fn farmsim_docs(index: usize) -> Canvas {
  let path = format!("C:\\Users\\Daggerbot\\{FARMSIM_DOCS}");
  let files = [
    Metadata::new_folder("inputDevices".to_string(), "18/05/2025 10:35".to_string()),
    Metadata::new_folder("mods".to_string(), "06/06/2025 12:20".to_string()),
    Metadata::new_folder("modSettings".to_string(), "04/06/2025 13:14".to_string()),
    Metadata::new_folder("music".to_string(), "17/12/2024 22:47".to_string()),
    Metadata::new_folder("pdlc".to_string(), "03/06/2025 19:52".to_string()),
    Metadata::new_folder("savegame1".to_string(), "25/12/2024 11:06".to_string()),
    Metadata::new_folder("savegame2".to_string(), "16/04/2025 15:34".to_string()),
    Metadata::new_folder("savegameBackup".to_string(), "16/04/2025 14:34".to_string()),
    Metadata::new_folder("screenshots".to_string(), "28/11/2024 08:09".to_string()),
    Metadata::new_folder("shader_cache".to_string(), "02/06/2025 07:00".to_string()),
    Metadata::new_folder("updater".to_string(), "22/03/2025 22:03".to_string()),
    Metadata::new_file("AHC_63805".to_string(), "27/11/2024 23:56".to_string(), "dat".to_string(), 2048),
    Metadata::new_file("AHT_63805".to_string(), "27/11/2024 23:56".to_string(), "dat".to_string(), 2048),
    Metadata::new_file("AVC_63805".to_string(), "27/11/2024 23:56".to_string(), "dat".to_string(), 2048),
    Metadata::new_file("AVD_63805".to_string(), "27/11/2024 23:56".to_string(), "dat".to_string(), 2048),
    Metadata::new_file("consoleHistory".to_string(), "16/04/2025 14:40".to_string(), "dat".to_string(), 7987),
    Metadata::new_file("extraContent".to_string(), "10/05/2025 09:16".to_string(), "xml".to_string(), 146),
    Metadata::new_file("game".to_string(), "01/06/2025 12:34".to_string(), "xml".to_string(), 1638),
    Metadata::new_file("gameSettings".to_string(), "01/06/2025 12:36".to_string(), "xml".to_string(), 6451),
    Metadata::new_file("IDT_63805".to_string(), "27/11/2025 23:57".to_string(), "dat".to_string(), 2048),
    Metadata::new_file("inputBinding".to_string(), "18/05/2025 10:35".to_string(), "xml".to_string(), 3072),
    Metadata::new_file("log".to_string(), "16/04/2025 15:36".to_string(), "txt".to_string(), 57344),
    Metadata::new_file("VERSION".to_string(), "27/05/2025 23:10".to_string(), "".to_string(), 7168)
  ];

  file_explorer(
    &path,
    &files,
    800,
    true,
    Some(index),
    Some(Style {
      theme: Theme::Dark,
      ..Default::default()
    })
  )
}
