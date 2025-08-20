#!/usr/bin/env python3
"""Add the final missing abstracts to benchmark_metrics.yaml"""

import yaml

# Load the current file
with open('benchmark_metrics.yaml', 'r') as f:
    data = yaml.safe_load(f)

# Final abstracts to add
final_papers = [
    {
        "title": "CondInst: Conditional Convolutions for Instance Segmentation",
        "algorithm": "CondInst",
        "abstract": "We propose a simple yet effective instance segmentation framework, termed CondInst (conditional convolutions for instance segmentation). Top-performing instance segmentation methods such as Mask R-CNN rely on ROI operations (typically ROIPool or ROIAlign) to obtain the final instance masks. In contrast, we propose to solve instance segmentation from a new perspective. Instead of using instance-wise ROIs as inputs to a network of fixed weights, we employ dynamic instance-aware networks, conditioned on instances. CondInst enjoys two advantages: 1) Instance segmentation is solved by a fully convolutional network, eliminating the need for ROI cropping and feature alignment. 2) Due to the much improved capacity of dynamically-generated conditional convolutions, the mask head can be very compact (e.g., 3 conv. layers, each having only 8 channels), leading to significantly faster inference. We demonstrate a simpler instance segmentation method that can achieve improved performance in both accuracy and inference speed. On the COCO dataset, we outperform a few recent methods including well-tuned Mask R-CNN baselines, without longer training schedules needed."
    },
    {
        "title": "EfficientNet: Rethinking Model Scaling for Convolutional Neural Networks",
        "algorithm": "EfficientNet",
        "abstract": "Convolutional Neural Networks (ConvNets) are commonly developed at a fixed resource budget, and then scaled up for better accuracy if more resources are available. In this paper, we systematically study model scaling and identify that carefully balancing network depth, width, and resolution can lead to better performance. Based on this observation, we propose a new scaling method that uniformly scales all dimensions of depth/width/resolution using a simple yet highly effective compound coefficient. We demonstrate the effectiveness of this method on scaling up MobileNets and ResNet. To go even further, we use neural architecture search to design a new baseline network and scale it up to obtain a family of models, called EfficientNets, which achieve much better accuracy and efficiency than previous ConvNets. In particular, our EfficientNet-B7 achieves state-of-the-art 84.3% top-1 accuracy on ImageNet, while being 8.4x smaller and 6.1x faster on inference than the best existing ConvNet. Our EfficientNets also transfer well and achieve state-of-the-art accuracy on CIFAR-100 (91.7%), Flowers (98.8%), and 3 other transfer learning datasets, with an order of magnitude fewer parameters."
    },
    {
        "title": "Grounding DINO: Marrying DINO with Grounded Pre-Training for Open-Set Object Detection",
        "algorithm": "Grounding DINO: Marrying DINO with Grounded Pre-Training for Open-Set Object Detection",
        "abstract": "In this paper, we present an open-set object detector, called Grounding DINO, by marrying Transformer-based detector DINO with grounded pre-training, which can detect arbitrary objects with human inputs such as category names or referring expressions. The key solution of open-set object detection is introducing language to a closed-set detector for open-set concept generalization. To effectively fuse language and vision modalities, we conceptually divide a closed-set detector into three phases and propose a tight fusion solution, which includes a feature enhancer, a language-guided query selection, and a cross-modality decoder for cross-modality fusion. While previous works mainly evaluate open-set object detection on novel categories, we propose to also perform evaluations on referring expression comprehension for objects specified with attributes. Grounding DINO performs remarkably well on all three settings, including benchmarks on COCO, LVIS, ODinW, and RefCOCO/+/g. Grounding DINO achieves a 52.5 AP on the COCO detection zero-shot transfer benchmark, i.e., without any training data from COCO. It sets a new record on the ODinW zero-shot benchmark with a mean 26.1 AP."
    }
]

# Add new papers to the data
data['papers'].extend(final_papers)

# Save the updated file
with open('benchmark_metrics.yaml', 'w') as f:
    yaml.dump(data, f, default_flow_style=False, sort_keys=False, allow_unicode=True, width=200)

print("Added final abstracts to benchmark_metrics.yaml")