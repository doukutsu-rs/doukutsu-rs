plugins {
    id("com.android.application")
}

// SDL's Java glue comes from the sources cargo checked out for sdl2-sys, the same ones CMake builds the native library from.
fun sdl2SourceDir(): File {
    val cargo = System.getenv("CARGO")
        ?: File(System.getProperty("user.home"), ".cargo/bin/cargo").takeIf { it.exists() }?.path
        ?: "cargo"
    val metadata = providers.exec {
        commandLine(cargo, "metadata", "--format-version", "1", "--manifest-path", file("../Cargo.toml").path)
    }.standardOutput.asText.get()

    @Suppress("UNCHECKED_CAST")
    val packages = (groovy.json.JsonSlurper().parseText(metadata) as Map<String, Any>)["packages"] as List<Map<String, Any>>
    val manifest = packages.first { it["name"] == "sdl2-sys" }["manifest_path"] as String
    return File(manifest).resolveSibling("SDL")
}

android {
    namespace = "io.github.doukutsu_rs"

    /**
     * NOTE: If you want to change the versions of packages required for the build
     * (e.g. build tools, compile SDK, NDK) or add new ones,
     * make the appropriate changes in packages.txt. Otherwise CI could break.
     */
    compileSdk = 35
    buildToolsVersion = "35.0.1"
    ndkVersion = "28.0.13004108"

    defaultConfig {
        applicationId = "io.github.doukutsu_rs"
        minSdk = 24
        targetSdk = 35
        versionCode = 3
        versionName = "1.0.0"

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"

        ndk {
            stl = "c++_shared"
        }

        externalNativeBuild {
            cmake {
                arguments.add("-DANDROID_STL=c++_shared")
            }
        }

        val documentsAuthorityValue = "$applicationId.documents"
        manifestPlaceholders["documentsAuthority"] = documentsAuthorityValue
        buildConfigField("String", "DOCUMENTS_AUTHORITY", "\"$documentsAuthorityValue\"")

        resValue("string", "app_name", "doukutsu-rs")
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )

            ndk {
                abiFilters.addAll(listOf("arm64-v8a", "armeabi-v7a", "x86", "x86_64"))
                stl = "c++_shared"
            }

            packaging {
                resources {
                    excludes.add("**/DebugProbesKt.bin")
                }
            }
        }
        debug {
            applicationIdSuffix = ".debug"
            resValue("string", "app_name", "doukutsu-rs (debug)")

            isJniDebuggable = true
            ndk {
                // ARM for testing on a physical device, and x86_64 for testing on the emulator
                abiFilters.addAll(listOf("arm64-v8a", "x86_64"))
            }

            val applicationId = defaultConfig.applicationId!!
            val documentsAuthorityValue = "$applicationId$applicationIdSuffix.documents"
            manifestPlaceholders["documentsAuthority"] = documentsAuthorityValue
            buildConfigField("String", "DOCUMENTS_AUTHORITY", "\"$documentsAuthorityValue\"")
        }
    }

    buildFeatures {
        buildConfig = true
    }

    sourceSets {
        getByName("main") {
            java.srcDirs(sdl2SourceDir().resolve("android-project/app/src/main/java"))
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    buildFeatures {
        viewBinding = true
    }

    externalNativeBuild {
        cmake {
            path = file("src/main/cpp/CMakeLists.txt")
        }
    }

    packaging {
        jniLibs {
            useLegacyPackaging = true
        }
    }
}

dependencies {
    implementation("androidx.annotation:annotation:1.5.0")
    implementation("androidx.appcompat:appcompat:1.6.0")
    implementation("androidx.constraintlayout:constraintlayout:2.1.1")
    implementation("androidx.core:core:1.9.0")
    implementation("com.google.android.material:material:1.8.0")
}
