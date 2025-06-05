struct Allpass {
    a: f32,
    x0: f32,
    x1: f32,
    x2: f32,

    y0: f32,
    y1: f32,
    y2: f32,
}

impl Default for Allpass {
    fn default() -> Self {
        Self {
            a: Default::default(),
            x0: Default::default(),
            x1: Default::default(),
            x2: Default::default(),
            y0: Default::default(),
            y1: Default::default(),
            y2: Default::default(),
        }
    }
}

impl Allpass {
    fn new(a: f32) -> Self {
        Self {
            a,
            ..Default::default()
        }
    }
    fn process(&mut self, input: f32) -> f32 {
        self.x2 = self.x1;
        self.x1 = self.x0;
        self.x0 = input;

        self.y2 = self.y1;
        self.y1 = self.y0;

        let output = self.x2 + ((input - self.y2) * self.a);

        self.y0 = output;
        output
    }
}

struct AllpassCascade {
    allpasses: Box<[Allpass]>,
}

impl AllpassCascade {
    fn process(&mut self, input: f32) -> f32 {
        let mut output = input;
        for filter in self.allpasses.iter_mut() {
            output = filter.process(output);
        }
        output
    }
}

struct PolyIIRHalfBandFilter {
    filter_a: AllpassCascade,
    filter_b: AllpassCascade,
    old_out: f32,
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum FilterType {
    Two,
    Four,
    Six,
    Eight,
    Ten,
    Twelve,
    SteepTwo,
    SteepFour,
    SteepSix,
    SteepEight,
    SteepTen,
    SteepTwelve,
}

impl FilterType {
    fn coefficients(&self) -> (Vec<f32>, Vec<f32>) {
        match self {
            FilterType::Two => (vec![0.23647102099689224], vec![0.7145421497126001]),
            FilterType::Four => (
                vec![0.07986642623635751, 0.5453536510711322],
                vec![0.28382934487410993, 0.8344118914807379],
            ),
            FilterType::Six => (
                vec![0.06029739095712437, 0.4125907203610563, 0.7727156537429234],
                vec![0.21597144456092948, 0.6043586264658363, 0.9238861386532906],
            ),
            FilterType::Eight => (
                vec![
                    0.03583278843106211,
                    0.2720401433964576,
                    0.5720571972357003,
                    0.827124761997324,
                ],
                vec![
                    0.1340901419430669,
                    0.4243248712718685,
                    0.7062921421386394,
                    0.9415030941737551,
                ],
            ),
            FilterType::Ten => (
                vec![
                    0.02366831419883467,
                    0.18989476227180174,
                    0.43157318062118555,
                    0.6632020224193995,
                    0.860015542499582,
                ],
                vec![
                    0.09056555904993387,
                    0.3078575723749043,
                    0.5516782402507934,
                    0.7652146863779808,
                    0.95247728378667541,
                ],
            ),
            FilterType::Twelve => (
                vec![
                    0.01677466677723562,
                    0.13902148819717805,
                    0.3325011117394731,
                    0.53766105314488,
                    0.7214184024215805,
                    0.8821858402078155,
                ],
                vec![
                    0.06501319274445962,
                    0.23094129990840923,
                    0.4364942348420355,
                    0.6329609551399348, //0.06329609551399348
                    0.80378086794111226,
                    0.9599687404800694,
                ],
            ),
            FilterType::SteepTwo => (vec![0.23647102099689224], vec![0.7145421497126001]),
            FilterType::SteepFour => (
                vec![0.12073211751675449, 0.6632020224193995],
                vec![0.3903621872345006, 0.890786832653497],
            ),
            FilterType::SteepSix => (
                vec![0.1271414136264853, 0.6528245886369117, 0.9176942834328115],
                vec![0.40056789819445626, 0.8204163891923343, 0.9763114515836773],
            ),
            FilterType::SteepEight => (
                vec![
                    0.07711507983241622,
                    0.4820706250610472,
                    0.7968204713315797,
                    0.9412514277740471,
                ],
                vec![
                    0.2659685265210946,
                    0.6651041532634957,
                    0.8841015085506159,
                    0.9820054141886075,
                ],
            ),
            FilterType::SteepTen => (
                vec![
                    0.051457617441190984,
                    0.35978656070567017,
                    0.6725475931034693,
                    0.8590884928249939,
                    0.9540209867860787,
                ],
                vec![
                    0.18621906251989334,
                    0.529951372847964,
                    0.7810257527489514,
                    0.9141815687605308,
                    0.985475023014907,
                ],
            ),
            FilterType::SteepTwelve => (
                vec![
                    0.036681502163648017,
                    0.2746317593794541,
                    0.56109896978791948,
                    0.769741833862266,
                    0.8922608180038789,
                    0.962094548378084,
                ],
                vec![
                    0.13654762463195771,
                    0.42313861743656667,
                    0.6775400499741616,
                    0.839889624849638,
                    0.9315419599631839,
                    0.9878163707328971,
                ],
            ),
        }
    }
}

impl PolyIIRHalfBandFilter {
    fn new(filter_type: FilterType) -> Self {
        let (a_coeff, b_coeff) = filter_type.coefficients();
        let a = a_coeff.into_iter().map(Allpass::new).collect();
        let filter_a = AllpassCascade { allpasses: a };
        let b = b_coeff.into_iter().map(Allpass::new).collect();
        let filter_b = AllpassCascade { allpasses: b };
        Self {
            filter_a,
            filter_b,
            old_out: 0.0,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let output = (self.filter_a.process(input) + self.old_out) * 0.5;
        self.old_out = self.filter_b.process(input);
        output
    }
}

pub struct Oversampler {
    oversampled: [f32; 2],
    interpolation_filter: PolyIIRHalfBandFilter,
    decimation_filter: PolyIIRHalfBandFilter,
}

impl Oversampler {
    pub fn new(filter_type: FilterType) -> Self {
        Self {
            oversampled: Default::default(),
            interpolation_filter: PolyIIRHalfBandFilter::new(filter_type),
            decimation_filter: PolyIIRHalfBandFilter::new(filter_type),
        }
    }
    pub fn interpolate(&mut self, input: f32) -> &mut [f32] {
        self.oversampled = [input, 0.0];
        for sample in self.oversampled.iter_mut() {
            *sample = self.interpolation_filter.process(*sample);
        }
        &mut self.oversampled
    }

    pub fn decimate(&mut self) -> f32 {
        self.decimation_filter.process(self.oversampled[0]);
        self.decimation_filter.process(self.oversampled[1])
    }
}
