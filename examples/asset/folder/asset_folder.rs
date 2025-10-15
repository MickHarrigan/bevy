//! Tries to load the `"examples/asset/folder/assets"` as individual assets

use std::{marker::PhantomData, path::PathBuf};

use bevy::prelude::*;
use bevy_asset::{AssetLoader, LoadDirectError};
use thiserror::Error;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                file_path: "examples/asset/folder/assets".to_string(),
                watch_for_changes_override: Some(true),
                // unapproved_path_mode: bevy_asset::UnapprovedPathMode::Allow,
                ..default()
            }),
            plugin,
        ))
        .add_systems(Startup, Items::setup)
        .add_systems(Update, update)
        .run();
}

fn plugin(app: &mut App) {
    app.init_asset::<Folder<Text>>()
        .init_asset::<Text>()
        .init_asset::<MyAssets>()
        .init_asset_loader::<FolderLoader<Text>>()
        .init_asset_loader::<TextLoader>()
        .init_asset_loader::<MyAssetLoader>();
}

#[derive(Asset, TypePath, Debug)]
struct Folder<A: Asset>(#[dependency] Vec<Handle<A>>);

struct FolderLoader<A>(PhantomData<A>);

impl<A> Default for FolderLoader<A> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<A: Asset> AssetLoader for FolderLoader<A> {
    type Asset = Folder<A>;
    type Settings = ();
    type Error = CommonError;

    async fn load(
        &self,
        _: &mut dyn bevy_asset::io::Reader,
        (): &Self::Settings,
        load_context: &mut bevy_asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let dir_path = load_context.path();
        let root_dir = std::env::var("CARGO_MANIFEST_DIR").map(PathBuf::from)?;

        let path = root_dir.join("examples/asset/folder/assets").join(dir_path);

        let mut collection = Vec::new();

        // Here is the messy part with `std::fs` reading
        for entry in std::fs::read_dir(path)? {
            let Ok(entry) = entry else {
                continue;
            };
            let path = entry.path();

            if load_context
                .loader()
                .immediate()
                .load::<A>(path.clone())
                .await
                .is_ok()
            {
                info!("Success: {:?} as {}", &path, A::type_path());
                let handle = load_context.load(path.clone());
                collection.push(handle);
            }
        }

        Ok(Folder(collection))
    }
}

#[derive(Asset, TypePath, Debug)]
#[expect(dead_code)]
struct Text(String);

#[derive(Default)]
struct TextLoader;

#[derive(Debug, Error)]
#[error(transparent)]
enum CommonError {
    Io(#[from] std::io::Error),
    Utf8(#[from] std::string::FromUtf8Error),
    Direct(#[from] LoadDirectError),

    Env(#[from] std::env::VarError),
}

impl AssetLoader for TextLoader {
    type Asset = Text;
    type Settings = ();
    type Error = CommonError;

    async fn load(
        &self,
        reader: &mut dyn bevy_asset::io::Reader,
        (): &Self::Settings,
        _: &mut bevy_asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let value = String::from_utf8(bytes)?;
        Ok(Text(value))
    }
}

#[derive(Asset, TypePath, Deref)]
struct MyAssets(Vec<Handle<Text>>);

#[derive(Default)]
struct MyAssetLoader;

impl AssetLoader for MyAssetLoader {
    type Asset = MyAssets;
    type Settings = ();
    type Error = CommonError;

    async fn load(
        &self,
        reader: &mut dyn bevy_asset::io::Reader,
        (): &Self::Settings,
        load_context: &mut bevy_asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        const WHICH: bool = true;
        if WHICH {
            // this method works
            FolderLoader(PhantomData)
                .load(reader, &(), load_context)
                .await
                .map(|folder| folder.0)
                .map_err(Into::into)
        } else {
            // This method fails (but gets the handles fine still)
            let path = load_context.path().to_path_buf();
            load_context
                .loader()
                .immediate()
                .load::<Folder<Text>>(path)
                .await
                .map(|asset| asset.take().0)
                .map_err(Into::into)
        }
        .map(MyAssets)
    }
}

#[derive(Resource, Deref)]
struct Items(Handle<MyAssets>);

impl Items {
    pub fn setup(mut commands: Commands, assets: Res<AssetServer>) {
        commands.insert_resource(Self(assets.load("primary")));
    }
}

fn update(
    items: Res<Items>,
    text_assets: Res<Assets<Text>>,
    asset_assets: Res<Assets<MyAssets>>,
    mut events: MessageReader<AssetEvent<MyAssets>>,
) {
    if !events.is_empty() {
        if let Some(assets) = asset_assets.get(&**items) {
            for handle in assets.iter() {
                let maybe_text = text_assets.get(handle);
                println!("Found text: {:?}", maybe_text);
            }
        }
        events.clear();
    }
}
