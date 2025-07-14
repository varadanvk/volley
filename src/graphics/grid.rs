use crate::graphics::vertex::Vertex;

pub fn create_grid_vertices(
    width: f32,
    height: f32,
    depth: f32,
    spacing: f32,
) -> (Vec<Vertex>, Vec<u16>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut index = 0u16;

    let grid_color = [0.3, 0.3, 0.3]; // Darker gray for 3D grid
    let accent_color = [0.5, 0.5, 0.5]; // Brighter for major grid lines

    // Calculate grid counts
    let lines_x = ((width / spacing) as i32) + 1;
    let lines_y = ((height / spacing) as i32) + 1;
    let lines_z = ((depth / spacing) as i32) + 1;

    // Generate YZ plane lines (vertical planes along X)
    for i in 0..lines_x {
        let x = -width / 2.0 + i as f32 * spacing;
        let color = if i % 5 == 0 { accent_color } else { grid_color };

        // Lines along Y at different Z positions
        for j in 0..lines_z {
            let z = -depth / 2.0 + j as f32 * spacing;

            vertices.push(Vertex {
                position: [x, -height / 2.0, z],
                color,
            });
            vertices.push(Vertex {
                position: [x, height / 2.0, z],
                color,
            });

            indices.push(index);
            indices.push(index + 1);
            index += 2;
        }

        // Lines along Z at different Y positions
        for j in 0..lines_y {
            let y = -height / 2.0 + j as f32 * spacing;

            vertices.push(Vertex {
                position: [x, y, -depth / 2.0],
                color,
            });
            vertices.push(Vertex {
                position: [x, y, depth / 2.0],
                color,
            });

            indices.push(index);
            indices.push(index + 1);
            index += 2;
        }
    }

    // Generate XZ plane lines (horizontal planes along Y)
    for i in 0..lines_y {
        let y = -height / 2.0 + i as f32 * spacing;
        let color = if i % 5 == 0 { accent_color } else { grid_color };

        // Lines along X at different Z positions
        for j in 0..lines_z {
            let z = -depth / 2.0 + j as f32 * spacing;

            vertices.push(Vertex {
                position: [-width / 2.0, y, z],
                color,
            });
            vertices.push(Vertex {
                position: [width / 2.0, y, z],
                color,
            });

            indices.push(index);
            indices.push(index + 1);
            index += 2;
        }

        // Lines along Z at different X positions
        for j in 0..lines_x {
            let x = -width / 2.0 + j as f32 * spacing;

            vertices.push(Vertex {
                position: [x, y, -depth / 2.0],
                color,
            });
            vertices.push(Vertex {
                position: [x, y, depth / 2.0],
                color,
            });

            indices.push(index);
            indices.push(index + 1);
            index += 2;
        }
    }

    // Generate XY plane lines (front/back planes along Z)
    for i in 0..lines_z {
        let z = -depth / 2.0 + i as f32 * spacing;
        let color = if i % 5 == 0 { accent_color } else { grid_color };

        // Lines along X at different Y positions
        for j in 0..lines_y {
            let y = -height / 2.0 + j as f32 * spacing;

            vertices.push(Vertex {
                position: [-width / 2.0, y, z],
                color,
            });
            vertices.push(Vertex {
                position: [width / 2.0, y, z],
                color,
            });

            indices.push(index);
            indices.push(index + 1);
            index += 2;
        }

        // Lines along Y at different X positions
        for j in 0..lines_x {
            let x = -width / 2.0 + j as f32 * spacing;

            vertices.push(Vertex {
                position: [x, -height / 2.0, z],
                color,
            });
            vertices.push(Vertex {
                position: [x, height / 2.0, z],
                color,
            });

            indices.push(index);
            indices.push(index + 1);
            index += 2;
        }
    }

    (vertices, indices)
}
