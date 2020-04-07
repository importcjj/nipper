use nipper::Document;

fn main() {
    let document = Document::from(
        r#"                <div class="loginContent">
    <div class="loginContentbg">
        <div class="el-dialog__wrapper login-dialog">
            <div role="dialog" aria-modal="true" aria-label="dialog"
                class="el-dialog el-dialog--center">
                <!---->
            </div>
        </div>
        <div class="el-dialog__wrapper login-dialog">
            <div role="dialog" aria-modal="true" aria-label="dialog"
                class="el-dialog el-dialog--center">
                <!---->
                <!---->
            </div>
        </div>
    </div>
</div>"#,
    );

    let mut div = document.select("div.loginContent");
    println!("{}", div.is("div"));

    println!("|{}|", div.text().trim());

    div.remove();

    println!("{}", document.html());
}
