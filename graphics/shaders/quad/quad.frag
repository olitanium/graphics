#version 330 core

in vec2 texture_coord;
out vec4 frag_colour;

uniform sampler2D in_texture;

void main() {  
    // Regular
    frag_colour = texture(in_texture, texture_coord);

    // Linear depth
    /*float far = 20.0;
    float near = 0.1;
    float ndc = texture(in_texture, texture_coord).r;
    float linear_depth = (2.0 * near * far) / (far + near - ndc * (far - near));	
    frag_colour = vec4(vec3(linear_depth/far), 1.0);*/

    // gray-scale
    /*frag_colour = texture(in_texture, texture_coord);
    float average = 0.2126 * frag_colour.r + 0.7152 * frag_colour.g + 0.0722 * frag_colour.b;
    frag_colour = vec4(average, average, average, 1.0);*/
    

    // Inverted
    // frag_colour = vec4( vec3(1.0 - texture(material.diffuse, texture_coord).rgb), 1.0);

    // Kernel 
    /*float offset_x = 1.0/500.0;
    float offset_y = 1.0/500.0;

    vec2 offsets[9] = vec2[](
        vec2(-offset_x,  offset_y), // top-left
        vec2( 0.0f,      offset_y), // top-center
        vec2( offset_x,  offset_y), // top-right
        vec2(-offset_x,  0.0f),   // center-left
        vec2( 0.0f,      0.0f),   // center-center
        vec2( offset_x,  0.0f),   // center-right
        vec2(-offset_x, -offset_y), // bottom-left
        vec2( 0.0f,     -offset_y), // bottom-center
        vec2( offset_x, -offset_y)  // bottom-right    
    );

    float blur[9] = float[](
        1.0 / 16, 2.0 / 16, 1.0 / 16,
        2.0 / 16, 4.0 / 16, 2.0 / 16,
        1.0 / 16, 2.0 / 16, 1.0 / 16  
    );

    float edge_detection[9] = float[](
        1.0,  1.0, 1.0,
        1.0, -8.0, 1.0,
        1.0,  1.0, 1.0  
    );
    
    float kernel[9] = edge_detection;

    vec3 sampleTex[9];
    for(int i = 0; i < 9; i++)
    {
        sampleTex[i] = vec3(texture(material.diffuse, texture_coord.st + offsets[i]));
    }
    vec3 col = vec3(0.0);
    for(int i = 0; i < 9; i++)
        col += sampleTex[i] * kernel[i];
    
    frag_colour = vec4(col, 1.0);
    
    float average = 0.2126 * frag_colour.r + 0.7152 * frag_colour.g + 0.0722 * frag_colour.b;
    
    if (average > 0.1) {
        frag_colour = vec4(vec3(1.0), 1.0);
    } else {
        frag_colour = vec4(vec3(0.0), 1.0);
    }
    */
}

