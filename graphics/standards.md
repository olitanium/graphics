# Standards

* create buffer id's as so:
```rust
    use std::ptr;
    let id = {
        let mut id = 0;
        unsafe { gl::function(..., &raw mut id, ...); }
        Wrapper(id)
    }
    
    unsafe_gl_may_error { gl::other(..., id.0, ...); }
```

* Doing shadows:
    
    list_lights =
        per light:
            make camera to represent the light
            use camera to make projtimesview matrix
            render scene from camera perspective onto framebuffer
            return (matrix, texture)
        return ListLights with (light, matrix and texture)

    ...


texture has name in shader_program
        has id
        has index in shader_program


in gpu: id -> index
        name -> index

        I choose the index


        When a name has the contents changed, the index can (and should) remain the same, but a new texture should be bound to that index.


textures are set by name

map (name -> (&Tex, Index))


new_list = Vec<(name, &Tex)>
old_list = Map<name -> &Tex>
name_to_index_list = Map<name -> (Index, &Tex)>

to add new texture:
    Is name in name_to_index_list?
    Yes:
        // It has been added before, so we can just bind the texture at the given index.
        old_list.set(name to &tex);
    No:
        add (name, tex) to new_list

to bind textures:
    // bind old textures
