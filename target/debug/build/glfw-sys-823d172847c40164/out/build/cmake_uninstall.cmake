
if (NOT EXISTS "/home/oliver/Documents/programming/rust/projects/graphics/target/debug/build/glfw-sys-823d172847c40164/out/build/install_manifest.txt")
  message(FATAL_ERROR "Cannot find install manifest: \"/home/oliver/Documents/programming/rust/projects/graphics/target/debug/build/glfw-sys-823d172847c40164/out/build/install_manifest.txt\"")
endif()

file(READ "/home/oliver/Documents/programming/rust/projects/graphics/target/debug/build/glfw-sys-823d172847c40164/out/build/install_manifest.txt" files)
string(REGEX REPLACE "\n" ";" files "${files}")

foreach (file ${files})
  message(STATUS "Uninstalling \"$ENV{DESTDIR}${file}\"")
  if (EXISTS "$ENV{DESTDIR}${file}")
    exec_program("/nix/store/dx4bdrs7mq3jfviqhszrc7l35ps9kg64-cmake-3.31.7/bin/cmake" ARGS "-E remove \"$ENV{DESTDIR}${file}\""
                 OUTPUT_VARIABLE rm_out
                 RETURN_VALUE rm_retval)
    if (NOT "${rm_retval}" STREQUAL 0)
      MESSAGE(FATAL_ERROR "Problem when removing \"$ENV{DESTDIR}${file}\"")
    endif()
  elseif (IS_SYMLINK "$ENV{DESTDIR}${file}")
    EXEC_PROGRAM("/nix/store/dx4bdrs7mq3jfviqhszrc7l35ps9kg64-cmake-3.31.7/bin/cmake" ARGS "-E remove \"$ENV{DESTDIR}${file}\""
                 OUTPUT_VARIABLE rm_out
                 RETURN_VALUE rm_retval)
    if (NOT "${rm_retval}" STREQUAL 0)
      message(FATAL_ERROR "Problem when removing symlink \"$ENV{DESTDIR}${file}\"")
    endif()
  else()
    message(STATUS "File \"$ENV{DESTDIR}${file}\" does not exist.")
  endif()
endforeach()

