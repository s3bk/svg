pub fn clock_val(s: &str) -> Result<f64> {
    alt((full_clock_val, partial_clock_val, timecount_val))(s)
}

fn full_clock_val(s: &str) -> Result<f64> {
    do_parse!(s,
        hours: digits >>
           tag(":") >>
        minutes: digits >>
            tag(":") >>
        seconds: float >>
        {
            hours * 3600. + minutes * 60. + seconds
        }
    );
}
fn partial_clock_val(s: &str) -> Result<f64> {
    do_parse!(s,
        minutes: digits >>
            tag(":") >>
        seconds: float >>
        {
            minutes * 60. + seconds
        }
    );
}