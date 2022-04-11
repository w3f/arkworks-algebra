use crate::hashing::HashToCurve;
use crate::{
    hashing::{
        curve_maps::{
            swu::{parity, SWUMap, SWUParams},
            wb::{WBMap, WBParams},
        },
        field_hashers::DefaultFieldHasher,
        map_to_curve_hasher::{MapToCurve, MapToCurveBasedHasher},
    },
    models::SWModelParameters,
    short_weierstrass_jacobian::GroupAffine,
    ModelParameters,
};
use ark_ff::{biginteger::BigInteger64, fields::Fp64, BigInt, MontBackend, MontFp};

use ark_ff::SquareRootField;
use ark_std::vec::Vec;
use ark_test_curves::bls12_381::{Fq as Fq_381, Fq2 as Fq2_381, Fq6 as Fq6_381, Fr as Fr_381, g1::Parameters as TestWBMapToCurveBLS12_381Params };
use hashbrown::HashMap;

#[cfg(all(test, feature = "std"))]
mod json;
#[cfg(all(test, feature = "std"))]
mod suites;

//Test for the implementation of parity function for various field extensions used in SWU hashing
#[test]
fn test_parity_of_prime_field_elements() {
    let a1 = Fq_381::from(0);
    let a2 = Fq_381::from(1);
    let a3 = Fq_381::from(10);
    assert_eq!(parity(&a1), false);
    assert_eq!(parity(&a2), true);
    assert_eq!(parity(&a3), false);
}

#[test]
fn test_parity_of_quadratic_extension_elements() {
    let element_test1 = Fq2_381::new(Fq_381::from(0), Fq_381::from(1));
    let element_test2 = Fq2_381::new(Fq_381::from(1), Fq_381::from(0));
    let element_test3 = Fq2_381::new(Fq_381::from(10), Fq_381::from(5));
    let element_test4 = Fq2_381::new(Fq_381::from(5), Fq_381::from(10));
    assert_eq!(parity(&element_test1), true, "parity is the oddness of first non-zero coefficient of element represented over the prime field" );
    assert_eq!(parity(&element_test2), true);
    assert_eq!(parity(&element_test3), false);
    assert_eq!(parity(&element_test4), true);
}

#[test]
fn test_parity_of_cubic_extension_elements() {
    let a1 = Fq2_381::new(Fq_381::from(0), Fq_381::from(0));
    let a2 = Fq2_381::new(Fq_381::from(0), Fq_381::from(1));
    let a3 = Fq2_381::new(Fq_381::from(1), Fq_381::from(0));
    let a4 = Fq2_381::new(Fq_381::from(1), Fq_381::from(1));
    let a5 = Fq2_381::new(Fq_381::from(0), Fq_381::from(2));

    let element_test1 = Fq6_381::new(a1, a2, a3);
    let element_test2 = Fq6_381::new(a2, a3, a4);
    let element_test3 = Fq6_381::new(a3, a4, a1);
    let element_test4 = Fq6_381::new(a4, a1, a2);
    let element_test5 = Fq6_381::new(a1, a5, a2);

    assert_eq!(parity(&element_test1), true, "parity is the oddness of first non-zero coefficient of element represented over the prime field");
    assert_eq!(parity(&element_test2), true, "parity is the oddness of first non-zero coefficient of element represented over the prime field");
    assert_eq!(parity(&element_test3), true);
    assert_eq!(parity(&element_test4), true);
    assert_eq!(parity(&element_test5), false);
}

//Tests for SWU hash to curve map for a curve on a toy example field of 127 elements
pub struct F127Config;
pub type F127 = Fp64<MontBackend<F127Config, 1>>;

impl ark_ff::MontConfig<1> for F127Config {
    // sage: FF(3)^63
    // 126
    #[rustfmt::skip]
    const TWO_ADIC_ROOT_OF_UNITY: F127 = MontFp!(F127, "126");

    /// MODULUS = 127
    #[rustfmt::skip]
    const MODULUS: BigInteger64 = BigInt!("127");

    // sage: FF(3).multiplicative_order()
    // 126
    // Montgomery conversion 3 * 2 = 6 % 127
    /// GENERATOR = 3
    #[rustfmt::skip]
    const GENERATOR: F127 = MontFp!(F127, "6");

    // T and T_MINUS_ONE_DIV_TWO, where MODULUS - 1 = 2^S * T
    // For T coprime to 2
}

const F127_ZERO: F127 = MontFp!(F127, "0");
const F127_ONE: F127 = MontFp!(F127, "1");

struct TestSWUMapToCurveParams;

impl ModelParameters for TestSWUMapToCurveParams {
    const COFACTOR: &'static [u64] = &[1];

    #[rustfmt::skip]
    const COFACTOR_INV: F127 = F127_ONE;

    type BaseField = F127;
    type ScalarField = F127;
}
/// just because not defining another field
///
/// from itertools import product
/// p = 127
/// FF = GF(p)
/// for a,b in product(range(0,p), range(0,p)):
///     try:
///         E = EllipticCurve([FF(a),FF(b)])
///         if E.order() == p:
///             print(E)
///     except:
///         pass
///
/// y^2 = x^3 + x + 63
impl SWModelParameters for TestSWUMapToCurveParams {
    /// COEFF_A = 1
    const COEFF_A: F127 = F127_ONE;

    /// COEFF_B = 1
    #[rustfmt::skip]
    const COEFF_B: F127 = MontFp!(F127, "63");

    /// AFFINE_GENERATOR_COEFFS = (G1_GENERATOR_X, G1_GENERATOR_Y)
    const AFFINE_GENERATOR_COEFFS: (Self::BaseField, Self::BaseField) =
        (MontFp!(F127, "62"), MontFp!(F127, "70"));
}

impl SWUParams for TestSWUMapToCurveParams {
    const XI: F127 = MontFp!(F127, "-1");
    const ZETA: F127 = MontFp!(F127, "3");
    const XI_ON_ZETA_SQRT: F127 = MontFp!(F127, "13");
}

/// test that MontFp make a none zero element out of 1
#[test]
fn test_field_element_construction() {
    let a1 = F127::from(1);
    let a2 = F127::from(2);
    let a3 = F127::from(125);

    assert!(F127::from(0) == a2 + a3);
    assert!(F127::from(0) == a2 * a1 + a3);
}

#[test]
fn test_field_division() {
    let num = F127::from(0x3d);
    let den = F127::from(0x7b);
    let num_on_den = F127::from(0x50);

    assert!(num / den == num_on_den);
}

/// Check that the hashing parameters are sane: zeta should be a non-square
#[test]
fn checking_the_hashing_parameters() {
    assert!(SquareRootField::legendre(&TestSWUMapToCurveParams::ZETA).is_qr() == false);
}

/// The point of the test is to get a simple SWU compatible curve and make
/// simple hash
#[test]
fn hash_arbitary_string_to_curve_swu() {
    use sha2::Sha256;

    let test_swu_to_curve_hasher = MapToCurveBasedHasher::<
        GroupAffine<TestSWUMapToCurveParams>,
        DefaultFieldHasher<Sha256, 128>,
        SWUMap<TestSWUMapToCurveParams>,
    >::new(&[1])
    .unwrap();

    let hash_result = test_swu_to_curve_hasher.hash(b"if you stick a Babel fish in your ear you can instantly understand anything said to you in any form of language.").expect("fail to hash the string to curve");

    assert!(
        hash_result.is_on_curve(),
        "hash results into a point off the curve"
    );
}

/// Use a simple SWU compatible curve and map the whole field to it. We observe
/// the map behaviour. Specifically, the map should be non-constant, all
/// elements should be mapped to curve successfully. everything can be mapped
#[test]
fn map_field_to_curve_swu() {
    let test_map_to_curve = SWUMap::<TestSWUMapToCurveParams>::new_map_to_curve().unwrap();

    let mut map_range: Vec<GroupAffine<TestSWUMapToCurveParams>> = vec![];
    for current_field_element in 0..127 {
        map_range.push(
            test_map_to_curve
                .map_to_curve(F127::from(current_field_element as u64))
                .unwrap(),
        );
    }

    let mut counts = HashMap::new();

    let mode = map_range
        .iter()
        .copied()
        .max_by_key(|&n| {
            let count = counts.entry(n).or_insert(0);
            *count += 1;
            *count
        })
        .unwrap();

    assert!(
        *counts.get(&mode).unwrap() != 127,
        "a constant hash function is not good."
    );
}

/// Testing WB19 hashing on a small curve
/// E_isogenous : Elliptic Curve defined by y^2 = x^3 + 109*x + 124 over Finite
/// Field of size 127
/// Isogenous to E : y^2 = x^3 + 3
struct TestSWU127MapToIsogenousCurveParams;

/// First we define the isogenous curve
/// sage: E_isogenous.order()
/// 127
impl ModelParameters for TestSWU127MapToIsogenousCurveParams {
    const COFACTOR: &'static [u64] = &[1];

    #[rustfmt::skip]
    const COFACTOR_INV: F127 = F127_ONE;

    type BaseField = F127;
    type ScalarField = F127;
}

/// E_isogenous : Elliptic Curve defined by y^2 = x^3 + 109*x + 124 over Finite
/// Field of size 127
impl SWModelParameters for TestSWU127MapToIsogenousCurveParams {
    /// COEFF_A = 109
    const COEFF_A: F127 = MontFp!(F127, "109");

    /// COEFF_B = 124
    #[rustfmt::skip]
    const COEFF_B: F127 = MontFp!(F127, "124");

    /// AFFINE_GENERATOR_COEFFS = (G1_GENERATOR_X, G1_GENERATOR_Y)
    const AFFINE_GENERATOR_COEFFS: (Self::BaseField, Self::BaseField) =
        (MontFp!(F127, "84"), MontFp!(F127, "2"));
}

/// SWU parameters for E_isogenous
impl SWUParams for TestSWU127MapToIsogenousCurveParams {
    /// NON-SQUARE = - 1
    const XI: F127 = MontFp!(F127, "-1");
    /// A Primitive Root of unity = 3
    const ZETA: F127 = MontFp!(F127, "3");
    /// sqrt(Xi/Zeta)
    const XI_ON_ZETA_SQRT: F127 = MontFp!(F127, "13");
}

/// The struct defining our parameters for the target curve of hashing
struct TestWBF127MapToCurveParams;

impl ModelParameters for TestWBF127MapToCurveParams {
    const COFACTOR: &'static [u64] = &[1];

    #[rustfmt::skip]
    const COFACTOR_INV: F127 = F127_ONE;

    type BaseField = F127;
    type ScalarField = F127;
}

/// E: Elliptic Curve defined by y^2 = x^3 + 3 over Finite
/// Field of size 127
impl SWModelParameters for TestWBF127MapToCurveParams {
    /// COEFF_A = 0
    const COEFF_A: F127 = F127_ZERO;

    /// COEFF_B = 3
    #[rustfmt::skip]
    const COEFF_B: F127 = MontFp!(F127, "3");

    /// AFFINE_GENERATOR_COEFFS = (G1_GENERATOR_X, G1_GENERATOR_Y)
    const AFFINE_GENERATOR_COEFFS: (Self::BaseField, Self::BaseField) =
        (MontFp!(F127, "62"), MontFp!(F127, "70"));
}

/// E_isogenous : Elliptic Curve defined by y^2 = x^3 + 109*x + 124 over Finite
/// Field of size 127
/// With psi: E_isogenous -> E
/// psi = (psi_x(x,y), psi_y(x,y))
/// where
/// psi_x: (-57*x^13 - 21*x^12 + 10*x^11 + 34*x^10 + 40*x^9 -
/// 13*x^8 + 32*x^7 - 32*x^6 + 23*x^5 - 14*x^4 + 39*x^3 + 23*x^2 + 63*x +
/// 4)/(x^12 - 13*x^11 + 11*x^10 - 33*x^9 - 30*x^8 + 30*x^7 + 34*x^6 - 44*x^5 +
/// 63*x^4 - 20*x^3 - 10*x^2 + 31*x + 2)
///
/// psi_y: (10*x^18*y + 59*x^17*y + 41*x^16*y + 48*x^15*y - 7*x^14*y + 6*x^13*y +
/// 5*x^12*y + 62*x^11*y + 12*x^10*y + 36*x^9*y - 49*x^8*y - 18*x^7*y - 63*x^6*y
/// - 43*x^5*y - 60*x^4*y - 18*x^3*y + 30*x^2*y - 57*x*y - 34*y)/(x^18 + 44*x^17
/// - 63*x^16 + 52*x^15 + 3*x^14 + 38*x^13 - 30*x^12 + 11*x^11 - 42*x^10 - 13*x^9
/// - 46*x^8 - 61*x^7 - 16*x^6 - 55*x^5 + 18*x^4 + 23*x^3 - 24*x^2 - 18*x + 32)
impl WBParams for TestWBF127MapToCurveParams {
    type IsogenousCurve = TestSWU127MapToIsogenousCurveParams;

    const PHI_X_NOM: &'static [<Self::IsogenousCurve as ModelParameters>::BaseField] = &[
        MontFp!(F127, "4"),
        MontFp!(F127, "63"),
        MontFp!(F127, "23"),
        MontFp!(F127, "39"),
        MontFp!(F127, "-14"),
        MontFp!(F127, "23"),
        MontFp!(F127, "-32"),
        MontFp!(F127, "32"),
        MontFp!(F127, "-13"),
        MontFp!(F127, "40"),
        MontFp!(F127, "34"),
        MontFp!(F127, "10"),
        MontFp!(F127, "-21"),
        MontFp!(F127, "-57"),
    ];

    const PHI_X_DEN: &'static [<Self::IsogenousCurve as ModelParameters>::BaseField] = &[
        MontFp!(F127, "2"),
        MontFp!(F127, "31"),
        MontFp!(F127, "-10"),
        MontFp!(F127, "-20"),
        MontFp!(F127, "63"),
        MontFp!(F127, "-44"),
        MontFp!(F127, "34"),
        MontFp!(F127, "30"),
        MontFp!(F127, "-30"),
        MontFp!(F127, "-33"),
        MontFp!(F127, "11"),
        MontFp!(F127, "-13"),
        MontFp!(F127, "1"),
    ];

    const PHI_Y_NOM: &'static [<Self::IsogenousCurve as ModelParameters>::BaseField] = &[
        MontFp!(F127, "-34"),
        MontFp!(F127, "-57"),
        MontFp!(F127, "30"),
        MontFp!(F127, "-18"),
        MontFp!(F127, "-60"),
        MontFp!(F127, "-43"),
        MontFp!(F127, "-63"),
        MontFp!(F127, "-18"),
        MontFp!(F127, "-49"),
        MontFp!(F127, "36"),
        MontFp!(F127, "12"),
        MontFp!(F127, "62"),
        MontFp!(F127, "5"),
        MontFp!(F127, "6"),
        MontFp!(F127, "-7"),
        MontFp!(F127, "48"),
        MontFp!(F127, "41"),
        MontFp!(F127, "59"),
        MontFp!(F127, "10"),
    ];

    const PHI_Y_DEN: &'static [<Self::IsogenousCurve as ModelParameters>::BaseField] = &[
        MontFp!(F127, "32"),
        MontFp!(F127, "-18"),
        MontFp!(F127, "-24"),
        MontFp!(F127, "23"),
        MontFp!(F127, "18"),
        MontFp!(F127, "-55"),
        MontFp!(F127, "-16"),
        MontFp!(F127, "-61"),
        MontFp!(F127, "-46"),
        MontFp!(F127, "-13"),
        MontFp!(F127, "-42"),
        MontFp!(F127, "11"),
        MontFp!(F127, "-30"),
        MontFp!(F127, "38"),
        MontFp!(F127, "3"),
        MontFp!(F127, "52"),
        MontFp!(F127, "-63"),
        MontFp!(F127, "44"),
        MontFp!(F127, "1"),
    ];
}

/// The point of the test is to get a simple WB compatible curve
/// and make simple hash
#[test]
fn hash_arbitary_string_to_curve_wb() {
    use sha2::Sha256;
    let test_wb_to_curve_hasher = MapToCurveBasedHasher::<
        GroupAffine<TestWBF127MapToCurveParams>,
        DefaultFieldHasher<Sha256, 128>,
        WBMap<TestWBF127MapToCurveParams>,
    >::new(&[1])
    .unwrap();

    let hash_result = test_wb_to_curve_hasher.hash(b"if you stick a Babel fish in your ear you can instantly understand anything said to you in any form of language.").expect("fail to hash the string to curve");

    assert!(
        hash_result.x != F127_ZERO && hash_result.y != F127_ZERO,
        "we assume that not both a and b coefficienst are zero for the test curve"
    );

    assert!(
        hash_result.is_on_curve(),
        "hash results into a point off the curve"
    );
}

// //////BLS12-381 Tests
//Tests for SWU hash to curve map for a curve isogenous to bls12-381 curve (G1)

pub struct TestSWUMapToIsoCurveBLS12_381Params;

impl ModelParameters for TestSWUMapToIsoCurveBLS12_381Params {
    type BaseField = Fq_381;
    type ScalarField = Fr_381;

    //sage: g1_iso.domain().order()/52435875175126190479447740508185965837690552500527637822603658699938581184513
    //76329603384216526031706109802092473003
    /// COFACTOR = (x - 1)^2 / 3  = 76329603384216526031706109802092473003
    const COFACTOR: &'static [u64] = &[0x8c00aaab0000aaab, 0x396c8c005555e156];

    /// COFACTOR_INV = COFACTOR^{-1} mod r
    /// = 52435875175126190458656871551744051925719901746859129887267498875565241663483
    #[rustfmt::skip]
    const COFACTOR_INV: Fr_381 = MontFp!(Fr_381, "52435875175126190458656871551744051925719901746859129887267498875565241663483");

}

impl SWModelParameters for TestSWUMapToIsoCurveBLS12_381Params {
     const COEFF_A: Fq_381 = MontFp!(Fq_381, "2858613208430792460670318198342879349494999260436483523154854961351063857243634726019465176474256126859776719994977");

    #[rustfmt::skip]
    const COEFF_B: Fq_381 = MontFp!(Fq_381, "2906670324641927570491258158026293881577086121416628140204402091718288198173574630967936031029026176254968826637280");

    /// AFFINE_GENERATOR_COEFFS = (G1_GENERATOR_X, G1_GENERATOR_Y)
    const AFFINE_GENERATOR_COEFFS: (Self::BaseField, Self::BaseField) =
        (MontFp!(Fq_381, "628127623378585612843095022119507708025289394540669560027004601611569871267541856210323712812047239723504248810248"), MontFp!(Fq_381, "344075650239127142968089520704786925624745533124141202280424406752399324209523628375922007963596482424831722220273"));

}

impl SWUParams for TestSWUMapToIsoCurveBLS12_381Params {
    const XI : Fq_381 = MontFp!(Fq_381, "11"); //a nonsquare in Fq ietf standard
    const ZETA: Fq_381 = MontFp!(Fq_381, "2"); //arbitatry primitive root of unity (element)
    const XI_ON_ZETA_SQRT: Fq_381 = MontFp!(Fq_381, "1496378135713580363480696149166996094826100595588415922929784991932724092840119474685212307129579508222230250924394"); ////square root of THETA=Xi/Zeta
 }

/// The point of the test is to get a  simpl SWU compatible curve
/// and make simple hash
#[test]
fn hash_arbitary_string_to_iso_curve_381_swu() {
    use sha2::Sha256;

    let test_swu_to_curve_hasher = MapToCurveBasedHasher::<GroupAffine<TestSWUMapToIsoCurveBLS12_381Params>, DefaultFieldHasher<Sha256, 128>, SWUMap<TestSWUMapToIsoCurveBLS12_381Params>>::new(&[1]).unwrap();
    
    let hash_result = test_swu_to_curve_hasher.hash(b"if you stick a Babel fish in your ear you can instantly understand anything said to you in any form of language.").expect("fail to hash the string to curve");    

    assert!(hash_result.x != MontFp!(Fq_381, "0"));

}

impl ModelParameters for TestWBMapToCurveBLS12_381Params {
    type BaseField = Fq_381;
    type ScalarField = Fr_381;

    /// COFACTOR = (x - 1)^2 / 3  = 76329603384216526031706109802092473003
    const COFACTOR: &'static [u64] = &[0x8c00aaab0000aaab, 0x396c8c005555e156];

    /// COFACTOR_INV = COFACTOR^{-1} mod r
    /// = 52435875175126190458656871551744051925719901746859129887267498875565241663483
    #[rustfmt::skip]
    const COFACTOR_INV: Fr_381 = MontFp!(Fr_381, "52435875175126190458656871551744051925719901746859129887267498875565241663483");
}

impl SWModelParameters for TestWBMapToCurveBLS12_381Params {
    /// COEFF_A = 0
    const COEFF_A: Fq_381 = MontFp!(Fq_381, "0");

    /// COEFF_B = 4
    #[rustfmt::skip]
    const COEFF_B: Fq_381 = MontFp!(Fq_381, "4");

    /// AFFINE_GENERATOR_COEFFS = (G1_GENERATOR_X, G1_GENERATOR_Y)
    const AFFINE_GENERATOR_COEFFS: (Self::BaseField, Self::BaseField) =
        (G1_GENERATOR_X, G1_GENERATOR_Y);

}

/// G1_GENERATOR_X =
/// 3685416753713387016781088315183077757961620795782546409894578378688607592378376318836054947676345821548104185464507
#[rustfmt::skip]
pub const G1_GENERATOR_X: Fq_381 = MontFp!(Fq_381, "3685416753713387016781088315183077757961620795782546409894578378688607592378376318836054947676345821548104185464507");

/// G1_GENERATOR_Y =
/// 1339506544944476473020471379941921221584933875938349620426543736416511423956333506472724655353366534992391756441569
#[rustfmt::skip]
pub const G1_GENERATOR_Y: Fq_381 = MontFp!(Fq_381, "1339506544944476473020471379941921221584933875938349620426543736416511423956333506472724655353366534992391756441569");

impl WBParams for TestWBMapToCurveBLS12_381Params
{
    type IsogenousCurve = TestSWUMapToIsoCurveBLS12_381Params;

    const PHI_X_NOM: &'static [<Self::IsogenousCurve as ModelParameters>::BaseField] = &[
        MontFp!(Fq_381, "3761822637321485742094536206199512035685972329360337092690555528605752326213440950527352563934445837165125977345128"), 
        MontFp!(Fq_381, "1582172990273894682725716146256297593851554078446457591584154223376480866715343525953458444978680215440412886996200"), 
        MontFp!(Fq_381, "2051387046688339481714726479723076305756384619135044672831882917686431912682625619320120082313093891743187631791280"), 
        MontFp!(Fq_381, "61316326124367244515865706164471217084261738749879925739220878889304439271692421994859529859373651892126645952478"), 
        MontFp!(Fq_381, "1424741831220874356476333227468129624471472782807764018784263716426284995285578915327628560152704910696985638070031"), 
        MontFp!(Fq_381, "3415427104483187489859740871640064348492611444552862448295571438270821994900526625562705192993481400731539293415811"), 
        MontFp!(Fq_381, "248188830067800966021611907001049410443171766148926993624301072093542166689921157756350157715209127315556469919811"), 
        MontFp!(Fq_381, "2458049485161426253398308320890830930555526088324701597510592431647721369610314802890725095474874074634194669518436"), 
        MontFp!(Fq_381, "1239271775787030039269460763652455868148971086016832054354147730155061349388626624328773377658494412538595239256855"), 
        MontFp!(Fq_381, "698396551686164607792478797181335970223204059946034999723234339823539961139901150842584755596191372859677741313422"), 
        MontFp!(Fq_381, "2756657124183929639940341559332863560985099912924783743137983687385942647530234634138642360072966950403354118194880"), 
        MontFp!(Fq_381, "1058488477413994682556770863004536636444795456512795473806825292198091015005841418695586811009326456605062948114985"),
    ];
    
    const PHI_X_DEN: &'static [<Self::IsogenousCurve as ModelParameters>::BaseField] = &[
        MontFp!(Fq_381, "3949438676361386880769263910006560135979986067074971624024178233787093666769860448538216069627988502376148329127381"), 
        MontFp!(Fq_381, "2822220997908397120956501031591772354860004534930174057793539372552395729721474912921980407622851861692773516917759"), 
        MontFp!(Fq_381, "610552060666338680048265043345785271200283516960578188721887711701602621050421759883463448407237338290466946893545"), 
        MontFp!(Fq_381, "2439329436782028399166625708751323253248871941520474623095864400521929929188290312899217468935910181179336068540275"), 
        MontFp!(Fq_381, "3025903087998593826923738290305187197829899948335370692927241015584233559365859980023579293766193297662657497834014"), 
        MontFp!(Fq_381, "2787329879967655380381218769366715121683732401785552165471280254930041329235866427760690206934082971711204373036769"), 
        MontFp!(Fq_381, "3729460208364303450659860043245117836792062818643913458168515950720008978604972045673944667221729675671707946923021"), 
        MontFp!(Fq_381, "3179090966864399634396993677377903383656908036827452986467581478509513058347781039562481806409014718357094150199902"), 
        MontFp!(Fq_381, "684141363729077267665842443966270070725528746651574246973574894998264196269884726340959705960779078660850485681497"), 
        MontFp!(Fq_381, "1355518942857092779104773143196445884975815408961178437135200875404433360418847982032652351120700883660623679118159"), 
        MontFp!(Fq_381, "1"),
    ];
    
    const PHI_Y_NOM: &'static [<Self::IsogenousCurve as ModelParameters>::BaseField] = &[
        MontFp!(Fq_381, "1393399195776646641963150658816615410692049723305861307490980409834842911816308830479576739332720113414154429643571"), 
        MontFp!(Fq_381, "1511190695657960398963160955727174407082178148587899660611396357887273149842318573217989398009716786569780748006283"), 
        MontFp!(Fq_381, "3614401657395238041642341964430562932885284129837437306094802414995585690467848276262207178353676484330879154111757"), 
        MontFp!(Fq_381, "303251954782077855462083823228569901064301365507057490567314302006681283228886645653148231378803311079384246777035"), 
        MontFp!(Fq_381, "3668073836642475590306080768959183669825119857553168245915502992504527111582288680993272182771099907781295865642364"), 
        MontFp!(Fq_381, "889147988023366972622306341891649433228352963186679791369365907311293432508530696403569262531255821940400079347315"), 
        MontFp!(Fq_381, "718493410301850496156792713845282235942975872282052335612908458061560958159410402177452633054233549648465863759602"), 
        MontFp!(Fq_381, "3406136881215263818934773248465685844790898101337646888092845387936582793628141067747381197872508534868489799639699"), 
        MontFp!(Fq_381, "331351138484847160772695176511692246547145983117580056167533060034011313825061073939286437522432693033324549699722"), 
        MontFp!(Fq_381, "2171468288973248519912068884667133903101171670397991979582205855298465414047741472281361964966463442016062407908400"), 
        MontFp!(Fq_381, "2092842804947501956457889650155528390118620253793900717919970123642236515732658355049092877407901587228724229223110"), 
        MontFp!(Fq_381, "3961890550435212003631878720741046102282781357458395684082291998816751808643768526367549852177588213715211235373916"), 
        MontFp!(Fq_381, "1707589313757812493102695021134258021969283151093981498394095062397393499601961942449581422761005023512037430861560"), 
        MontFp!(Fq_381, "1967501030954114997918812128990512798136077044765138085902549248681331523022349398587390395919802044865687181234417"), 
        MontFp!(Fq_381, "2922895688438869655803185556286420403397802691723657346548307498540648684066474272936979722182960006715481007746439"), 
        MontFp!(Fq_381, "3370924952219000111210625390420697640496067348723987858345031683392215988129398381698161406651860675722373763741188"),
    ];
    
    const PHI_Y_DEN: &'static [<Self::IsogenousCurve as ModelParameters>::BaseField] = &[
        MontFp!(Fq_381, "3396434800020507717552209507749485772788165484415495716688989613875369612529138640646200921379825018840894888371137"), 
        MontFp!(Fq_381, "3955937245707125245654875920344947126613343076099634862876498453376019064171085472617926084668015244086355452990926"), 
        MontFp!(Fq_381, "3115801080798375384198119675359305198682407447857559059827677597200837822471239996878963936531572341910295439078930"), 
        MontFp!(Fq_381, "3496628876382137961119423566187258795236027183112131017519536056628828830323846696121917502443333849318934945158166"), 
        MontFp!(Fq_381, "3819161294135653749488194485080848928281288158623143986707267064691105653307717275066596854814679462226419882338445"), 
        MontFp!(Fq_381, "3838344850882296394939911407571260624820518547352197287854517462270146783414326660145944161947898461843919024387456"), 
        MontFp!(Fq_381, "3443845896188810583748698342858554856823966611538932245284665132724280883115455093457486044009395063504744802318172"), 
        MontFp!(Fq_381, "1189518655724056699355159955938426141594501095232997073362878098547301776615800227099534760471261853454585261625212"), 
        MontFp!(Fq_381, "2309935917761931164183799788994195243166135866798971442738101234750394172839060646960604788222384454747599044244610"), 
        MontFp!(Fq_381, "3459661102222301807083870307127272890283709299202626530836335779816726101522661683404130556379097384249447658110805"), 
        MontFp!(Fq_381, "3898950452492751420431682235553362135160293985934212849206350896691847297403795877806915602445207834970900272834330"), 
        MontFp!(Fq_381, "2381968764806454673720955440620228057919683152985846777578004069901421179563597288802750793515266450448328892335136"), 
        MontFp!(Fq_381, "1668238650112823419388205992952852912407572045257706138925379268508860023191233729074751042562151098884528280913356"), 
        MontFp!(Fq_381, "939263815598411948154903329612610969504805630387130456691588334780964163089645546614263028349325528011160458395367"), 
        MontFp!(Fq_381, "32073636674805471948264801926716749185281703472263713036772245044634215382853040827634712116543493471988382397345"), 
        MontFp!(Fq_381, "1"),
    ];
}

#[test]
fn hash_arbitary_string_to_bls12_381_curve_wb() {
    use sha2::Sha256;
    let test_wb_to_curve_hasher = MapToCurveBasedHasher::<
        GroupAffine<TestWBMapToCurveBLS12_381Params>,
        DefaultFieldHasher<Sha256, 128>,
        WBMap<TestWBMapToCurveBLS12_381Params>,
    >::new(&[1])
    .unwrap();

    let hash_result = test_wb_to_curve_hasher.hash(b"if you stick a Babel fish in your ear you can instantly understand anything said to you in any form of language.").expect("fail to hash the string to curve");

    assert!(
        hash_result.x != MontFp!(Fq_381, "0") && hash_result.y != MontFp!(Fq_381, "0"),
        "we assume that not both a and b coefficienst are zero for the test curve"
    );

    assert!(
        hash_result.is_on_curve(),
        "hash results into a point off the curve"
    );
}
