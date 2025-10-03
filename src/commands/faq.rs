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
        MetadataType,
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
static FARMSIM_DOCS_MSFT: &str = "%LocalAppData%\\Packages\\GIANTSSoftware.FarmingSimulator25PC_fa8jxm5fj0esw\\LocalCache\\Local";

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
  FsEnableDevConsole,
  #[name = "[FS] Profile path for Microsoft Store copy"]
  FsMsftStoreProfile
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
      let img = farmsim_docs(10).to_bytes(Some(ImageFormat::Jpeg { quality: 100 }))?;

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
      let img = farmsim_docs(22).to_bytes(Some(ImageFormat::Jpeg { quality: 100 }))?;

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
    },
    Questions::FsMsftStoreProfile => {
      let embed = build_faq(
        "Locating your profile on Microsoft Store copy",
        format!(
          "You can find your game's profile directory at `{FARMSIM_DOCS_MSFT}`, this location choice is not Giants' fault as other games on \
           Microsoft Store also shares the same `Packages` directory.\nYou can install mods here or modify `game.xml` file or anything else as this \
           is the identical structure as any other copies on different platforms like Steam and Epic Games."
        ),
        None
      );
      ctx.send(CreateReply::default().embed(embed)).await?;
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

  if let Some(img) = image {
    embed = embed.image(format!("attachment://{img}"));
  }

  embed
}

fn farmsim_docs(index: usize) -> Canvas {
  let path = format!("C:\\Users\\Daggerbot\\{FARMSIM_DOCS}");
  let items = [
    MetadataType::Folder {
      name: "inputDevices",
      date: "18/05/2025 10:35"
    },
    MetadataType::Folder {
      name: "mods",
      date: "06/06/2025 12:20"
    },
    MetadataType::Folder {
      name: "modSettings",
      date: "04/06/2025 13:14"
    },
    MetadataType::Folder {
      name: "music",
      date: "17/12/2024 22:47"
    },
    MetadataType::Folder {
      name: "pdlc",
      date: "03/06/2025 19:52"
    },
    MetadataType::Folder {
      name: "savegame1",
      date: "25/12/2024 11:06"
    },
    MetadataType::Folder {
      name: "savegame2",
      date: "16/04/2025 15:34"
    },
    MetadataType::Folder {
      name: "savegame3",
      date: "01/10/2025 04:30"
    },
    MetadataType::Folder {
      name: "savegameBackup",
      date: "16/04/2025 14:34"
    },
    MetadataType::Folder {
      name: "screenshots",
      date: "28/11/2024 08:09"
    },
    MetadataType::Folder {
      name: "shader_cache",
      date: "01/10/2025 03:32"
    },
    MetadataType::Folder {
      name: "updater",
      date: "27/09/2025 20:00"
    },
    MetadataType::File {
      name:      "AHC_63805",
      date:      "27/11/2024 23:56",
      extension: "dat",
      size:      2048
    },
    MetadataType::File {
      name:      "AHT_63805",
      date:      "27/11/2024 23:56",
      extension: "dat",
      size:      2048
    },
    MetadataType::File {
      name:      "AVC_63805",
      date:      "27/11/2024 23:56",
      extension: "dat",
      size:      2048
    },
    MetadataType::File {
      name:      "AVD_63805",
      date:      "27/11/2024 23:56",
      extension: "dat",
      size:      2048
    },
    MetadataType::File {
      name:      "consoleHistory",
      date:      "29/09/2025 15:59",
      extension: "dat",
      size:      7992
    },
    MetadataType::File {
      name:      "extraContent",
      date:      "10/05/2025 09:16",
      extension: "xml",
      size:      146
    },
    MetadataType::File {
      name:      "game",
      date:      "01/10/2025 03:26",
      extension: "xml",
      size:      1638
    },
    MetadataType::File {
      name:      "gameSettings",
      date:      "01/10/2025 03:26",
      extension: "xml",
      size:      6451
    },
    MetadataType::File {
      name:      "IDT_63805",
      date:      "27/11/2025 23:57",
      extension: "dat",
      size:      2048
    },
    MetadataType::File {
      name:      "inputBinding",
      date:      "18/05/2025 10:35",
      extension: "xml",
      size:      3072
    },
    MetadataType::File {
      name:      "log",
      date:      "01/10/2025 04:30",
      extension: "txt",
      size:      5242880
    },
    MetadataType::File {
      name:      "VERSION",
      date:      "27/09/2025 19:59",
      extension: "",
      size:      8
    }
  ];

  file_explorer(
    &path,
    &Metadata::bulk_new(&items),
    800,
    true,
    Some(index),
    Some(Style {
      theme: Theme::Dark,
      ..Default::default()
    })
  )
}
