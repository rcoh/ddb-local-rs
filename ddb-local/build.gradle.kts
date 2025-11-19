plugins {
    java
    // Executes smithy-build process to generate server stubs
    id("software.amazon.smithy.gradle.smithy-base")
}

description = "DynamoDB Local server implementation"

dependencies {
    val smithyRsVersion: String by project

    // === Code generators ===
    smithyBuild("software.amazon.smithy.rust:codegen-server:$smithyRsVersion")

    // === Service model ===
    implementation(project(":smithy"))

}

tasks.register<Copy>("copyGeneratedCode") {
    dependsOn("smithyBuild")
    from("build/smithyprojections/ddb-local/source/rust-server-codegen")
    into("../server-sdk")
}

tasks.named("build") {
    finalizedBy("copyGeneratedCode")
}

