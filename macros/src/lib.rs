use proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro_derive(BindUniform)]
pub fn bind_uniform_derive(input: TokenStream) -> TokenStream {
  let ast: syn::DeriveInput = syn::parse(input).unwrap();
  let ident = &ast.ident;

  let data = if let syn::Data::Struct(data) = &ast.data {
    data
  } else {
    unimplemented!()
  };

  let fields = if let syn::Fields::Named(fields) = &data.fields {
    fields
  } else {
    unimplemented!()
  };

  let calls = fields
    .named
    .iter()
    .map(|field| {
      let ident = field.ident.as_ref().unwrap();
      let ident_str = ident.to_string();
      quote! {
        self
          .#ident
          .bind_uniform(gl, shader, &format!("{}.{}", name, #ident_str));
      }
    })
    .collect::<Vec<_>>();

  let imp = quote! {
    impl crate::shader::BindUniform for #ident {
      unsafe fn bind_uniform(&self, gl: &glow::Context, shader: &mut crate::shader::ActiveShader, name: &str) {
        #(#calls)*
      }
    }
  };

  imp.into()
}

#[proc_macro_derive(ShaderTypeDef)]
pub fn bind_shader_type_def(input: TokenStream) -> TokenStream {
  let ast: syn::DeriveInput = syn::parse(input).unwrap();
  let ident = &ast.ident;

  let data = if let syn::Data::Struct(data) = &ast.data {
    data
  } else {
    unimplemented!()
  };

  let fields = if let syn::Fields::Named(fields) = &data.fields {
    fields
  } else {
    unimplemented!()
  };

  let shader_fields = fields
    .named
    .iter()
    .map(|field| {
      let ident = field.ident.as_ref().unwrap();
      let ident_str = ident.to_string();

      let extract_segment = |ty: &syn::Type| {
        if let syn::Type::Path(path) = ty {
          path.path.segments[0].clone()
        } else {
          unimplemented!()
        }
      };

      let ty = extract_segment(&field.ty);

      let primitive_type = |ident: &syn::Ident| match ident.to_string().as_str() {
        "f32" => "float",
        "Vec4" => "vec4",
        "Vec3" => "vec3",
        "Vec2" => "vec2",
        "Mat4" => "mat4",
        "T2d" | "Texture" => "sampler2D",
        "TCubemap" => "samplerCube",
        id => unimplemented!("primtive {}", id),
      };

      match &ty.arguments {
        syn::PathArguments::AngleBracketed(args) => {
          let arg = if let syn::GenericArgument::Type(arg) = args.args.first().unwrap() {
            extract_segment(arg)
          } else {
            unimplemented!()
          };

          match ty.ident.to_string().as_str() {
            "Vec" => {
              format!(
                "{} {}[4]; int {}_len;",
                primitive_type(&arg.ident),
                ident_str,
                ident_str
              )
            }
            "Texture" => {
              format!("{} {};", primitive_type(&arg.ident), ident_str)
            }
            _ => unimplemented!(),
          }
        }
        syn::PathArguments::None => {
          format!("{} {};", primitive_type(&ty.ident), ident_str)
        }
        _ => unimplemented!(),
      }
    })
    .collect::<Vec<_>>();

  let type_def = format!(
    r#"struct {} {{ 
  {} 
}};"#,
    ident.to_string(),
    shader_fields.join("\n")
  );

  let imp = quote! {
    impl crate::shader::ShaderTypeDef for #ident {
      const TYPE_DEF: &'static str = #type_def;
    }
  };

  imp.into()
}

#[proc_macro_derive(ShaderBlockDef)]
pub fn bind_shader_block_def(input: TokenStream) -> TokenStream {
  let ast: syn::DeriveInput = syn::parse(input).unwrap();
  let ident = &ast.ident;

  let data = if let syn::Data::Struct(data) = &ast.data {
    data
  } else {
    unimplemented!()
  };

  let fields = if let syn::Fields::Named(fields) = &data.fields {
    fields
  } else {
    unimplemented!()
  };

  let shader_fields = fields
    .named
    .iter()
    .map(|field| {
      let ident = field.ident.as_ref().unwrap();
      let ty = if let syn::Type::Path(path) = &field.ty {
        path.path.segments[1].ident.to_string()
      } else {
        unimplemented!()
      };

      format!("{} {};", ty, ident.to_string())
    })
    .collect::<Vec<_>>();

  let type_def = format!(
    r#"layout (std140) uniform {} {{ 
  {} 
}};"#,
    ident.to_string(),
    shader_fields.join("\n")
  );

  let imp = quote! {
    impl crate::shader::ShaderBlockDef for #ident {
      const BLOCK_DEF: &'static str = #type_def;
    }
  };

  imp.into()
}
