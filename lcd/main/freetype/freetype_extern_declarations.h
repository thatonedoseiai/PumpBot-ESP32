//--Typedefs--
typedef signed int FT_Int32;
typedef unsigned int FT_UInt32;
typedef unsigned char  FT_Bool;
typedef signed short  FT_FWord;
typedef unsigned short  FT_UFWord;
typedef signed char  FT_Char;
typedef unsigned char  FT_Byte;
typedef const FT_Byte*  FT_Bytes;
typedef FT_UInt32  FT_Tag;
typedef char  FT_String;
typedef signed short  FT_Short;
typedef unsigned short  FT_UShort;
typedef signed int  FT_Int;
typedef unsigned int  FT_UInt;
typedef signed long  FT_Long;
typedef unsigned long  FT_ULong;
typedef signed short  FT_F2Dot14;
typedef signed long  FT_F26Dot6;
typedef signed long  FT_Fixed;
typedef int  FT_Error;
typedef void*  FT_Pointer;
typedef size_t  FT_Offset;

//--Structs--
typedef struct  FT_FaceRec_ {
    FT_Long num_faces;
    FT_Long face_index;
    FT_Long face_flags;
    FT_Long style_flags;
    FT_Long num_glyphs;
    FT_String family_name;
    FT_String style_name;
    FT_Int num_fixed_sizes;
    FT_Bitmap_Size* available_sizes;
    FT_Int num_charmaps;
    FT_CharMap* charmaps;
    FT_Generic generic;
    FT_BBox bbox;
    FT_UShort units_per_EM;
    FT_Short ascender;
    FT_Short descender;
    FT_Short height;
    FT_Short max_advance_width;
    FT_Short max_advance_height;
    FT_Short underline_position;
    FT_Short underline_thickness;
    FT_GlyphSlot glyph;
    FT_Size size;
    FT_CharMap charmap;
    FT_Driver driver;
    FT_Memory memory;
    FT_Stream stream;
    FT_ListRec sizes_list;
    FT_Generic autohint;
    void* extensions;
    FT_Face_Internal internal;
  } FT_FaceRec;
  typedef struct  FT_Bitmap_ {
    unsigned int rows;
    unsigned int width;
    int pitch;
    unsigned char* buffer;
    unsigned short num_grays;
    unsigned char pixel_mode;
    unsigned char palette_mode;
    void* palette;

  } FT_Bitmap;
  typedef struct  FT_Open_Args_ {
    FT_UInt flags;
    const FT_Byte* memory_base;
    FT_Long memory_size;
    FT_String* pathname;
    FT_Stream stream;
    FT_Module driver;
    FT_Int num_params;
    FT_Parameter* params;
  } FT_Open_Args;
typedef struct FT_MemoryRec_* FT_Memory;
	struct  FT_MemoryRec_ {
		void*  user;
		FT_Alloc_Func alloc;
		FT_Free_Func free;
		FT_Realloc_Func realloc;
	};
	typedef struct  FT_Parameter_ {
		FT_ULong    tag;
		FT_Pointer  data;
	} FT_Parameter;
typedef struct FT_LibraryRec_ *FT_Library; //TODO: FIX!

//--Functions--
extern int FT_Init_FreeType(FT_Library *alibrary);
extern int FT_New_Face(FT_Library library, const char* filepathname, FT_Long face_index, FT_Face *aface);
extern int FT_Open_Face(FT_Library library, const FT_Open_Args* args, FT_Long face_index, FT_Face *aface);
extern int FT_Set_Char_Size(FT_Face face, FT_F26Dot6 char_width, FT_F26Dot6 char_height, FT_UInt horz_resolution, FT_UInt vert_resolution);
extern int FT_Load_Char(FT_Face face, FT_ULong char_code, FT_Int32 load_flags);