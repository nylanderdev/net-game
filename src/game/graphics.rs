use ggez::graphics::{Color, DrawMode, Mesh, MeshBuilder, Rect};
use ggez::nalgebra::Point2;
use ggez::{Context, GameResult};

const DEFAULT_MESH: MeshGenerator = bullet_mesh;

pub type MeshGenerator = fn(ctx: &mut Context, x: f32, y: f32, color: Color) -> GameResult<Mesh>;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum MeshType {
    Default,
    Tank,
    Bullet,
    Wall,
    Heal,
    None,
}

impl Default for MeshType {
    fn default() -> Self {
        Self::Default
    }
}

pub fn generator_from_mesh_type(mesh_type: MeshType) -> MeshGenerator {
    match mesh_type {
        MeshType::Tank => tank_mesh,
        MeshType::Bullet => bullet_mesh,
        MeshType::Wall => wall_mesh,
        MeshType::Heal => heal_item_mesh,
        MeshType::Default | _ => DEFAULT_MESH
    }
}

pub fn tank_mesh(ctx: &mut Context, x: f32, y: f32, color: Color) -> GameResult<Mesh> {
    MeshBuilder::new()
        .circle(
            DrawMode::fill(),
            Point2::new(x /*+ 25.0*/, y /*+ 25.0*/),
            25.0,
            0.1,
            color,
        )
        .rectangle(
            DrawMode::fill(),
            Rect::new(x /*+ 25.0*/, y - 15.0 /*+ 10.0*/, 30.0, 30.0),
            color,
        )
        .build(ctx)
}

pub fn bullet_mesh(ctx: &mut Context, x: f32, y: f32, color: Color) -> GameResult<Mesh> {
    const RADIUS: f32 = 5.0;
    MeshBuilder::new()
        .circle(
            DrawMode::fill(),
            Point2::new(x /*+ 2.0 * RADIUS*/, y /*+ RADIUS*/),
            RADIUS,
            0.1,
            color,
        )
        .rectangle(
            DrawMode::fill(),
            Rect::new(x - 2.0 * RADIUS, y - RADIUS, 2.0 * RADIUS, 2.0 * RADIUS),
            color,
        )
        .build(ctx)
}

pub fn wall_mesh(ctx: &mut Context, x: f32, y: f32, color: Color) -> GameResult<Mesh> {
    MeshBuilder::new()
        .rectangle(
            DrawMode::fill(),
            Rect::new(x, y, 1.0, 1.0),
            color,
        )
        .build(ctx)
}

pub fn inventory_mesh(ctx: &mut Context, x: f32, y: f32) -> GameResult<Mesh> {
    let mut builder = MeshBuilder::new();
    const INVENTORY_SIZE: f32 = 40.0;
    builder.rectangle(
        DrawMode::fill(),
        Rect::new(x, y, INVENTORY_SIZE, INVENTORY_SIZE),
        Color::new(0.0, 0.0, 0.0, 0.25),
    );
    builder.build(ctx)
}

pub fn heal_item_mesh(ctx: &mut Context, x: f32, y: f32, color: Color) -> GameResult<Mesh> {
    let mut builder = MeshBuilder::new();
    const ITEM_SIZE: f32 = 30.0;
    const ITEM_BOLDNESS: f32 = 10.0;
    builder.rectangle(
        DrawMode::fill(),
        Rect::new(x + 0.5 * (ITEM_SIZE - ITEM_BOLDNESS), y,
                  ITEM_BOLDNESS, ITEM_SIZE),
        color.clone(),
    );
    builder.rectangle(
        DrawMode::fill(),
        Rect::new(x, y + 0.5 * (ITEM_SIZE - ITEM_BOLDNESS),
                  ITEM_SIZE, ITEM_BOLDNESS),
        color,
    );
    builder.build(ctx)
}

pub fn health_bar(
    ctx: &mut Context,
    x: f32,
    y: f32,
    color: Color,
    health: u8,
    total_health: u8,
) -> GameResult<Mesh> {
    const HORIZONTAL_MARGIN: f32 = 5.0;
    const VERTICAL_MARGIN: f32 = 5.0;
    const SPACING: f32 = 0.0;
    const HEALTH_HEIGHT: f32 = 40.0;
    const HEALTH_WIDTH: f32 = 5.0;
    const BAR_HEIGHT: f32 = 2.0 * VERTICAL_MARGIN + HEALTH_HEIGHT;
    let mut bar_width =
        HORIZONTAL_MARGIN + (total_health as f32) * (HEALTH_WIDTH + SPACING) + VERTICAL_MARGIN;
    if total_health > 0 {
        bar_width -= SPACING;
    }
    let mut builder = MeshBuilder::new();
    builder.rectangle(
        DrawMode::fill(),
        Rect::new(x, y, bar_width, BAR_HEIGHT),
        Color::new(0.0, 0.0, 0.0, 0.25),
    );
    for i in 0..health {
        builder.rectangle(
            DrawMode::fill(),
            Rect::new(
                x + HORIZONTAL_MARGIN + (i as f32) * (HEALTH_WIDTH + SPACING),
                y + VERTICAL_MARGIN,
                HEALTH_WIDTH,
                HEALTH_HEIGHT,
            ),
            color,
        );
    }
    builder.build(ctx)
}